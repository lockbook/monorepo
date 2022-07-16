use crate::access_info::{EncryptedFolderAccessKey, UserAccessInfo};
use crate::account::{Account, Username};
use crate::crypto::AESKey;
use crate::file_like::FileLike;
use crate::file_metadata::{FileType, Owner};
use crate::secret_filename::SecretFileName;
use crate::tree_like::TreeError::*;
use crate::{pubkey, symkey};
use core::fmt;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use uuid::Uuid;

pub trait TreeLike {
    type F: FileLike;

    fn ids(&self) -> HashSet<Uuid>;
    fn maybe_find(&self, id: Uuid) -> Option<&Self::F>;

    fn find(&self, id: Uuid) -> Result<&Self::F, TreeError> {
        self.maybe_find(id).ok_or(FileNonexistent)
    }

    fn maybe_find_parent<F2: FileLike>(&self, file: &F2) -> Option<&Self::F> {
        self.maybe_find(file.parent())
    }

    fn find_parent<F2: FileLike>(&self, file: &F2) -> Result<&Self::F, TreeError> {
        self.maybe_find_parent(file).ok_or(FileParentNonexistent)
    }

    fn stage<'a, Staged>(&'a self, staged: &'a Staged) -> StagedTree<'a, Self, Staged>
    where
        Staged: TreeLike,
        Self: Sized,
    {
        StagedTree { base: self, local: staged }
    }
}

impl<F: FileLike> TreeLike for [F] {
    type F = F;

    fn ids(&self) -> HashSet<Uuid> {
        self.iter().map(|f| f.id()).collect()
    }

    fn maybe_find<'a>(&'a self, id: Uuid) -> Option<&'a Self::F> {
        self.iter().find(|f| f.id() == id)
    }
}

pub struct LazyTree<T: TreeLike> {
    tree: T,
    name_by_id: HashMap<Uuid, String>,
    key_by_id: HashMap<Uuid, AESKey>,
    implicitly_deleted_by_id: HashMap<Uuid, bool>,
}

impl<T: TreeLike> LazyTree<T> {
    pub fn new(tree: T) -> Self {
        Self {
            tree,
            name_by_id: HashMap::new(),
            key_by_id: HashMap::new(),
            implicitly_deleted_by_id: HashMap::new(),
        }
    }
}

impl<'a, T: TreeLike> LazyTree<T> {
    pub fn calculate_deleted(&mut self, id: Uuid) -> Result<bool, TreeError> {
        let (visited_ids, deleted) = {
            let mut file = self.find(id)?;
            let mut visited_ids = vec![];
            let mut deleted = false;

            while !file.is_root() {
                visited_ids.push(file.id());
                if let Some(&implicit) = self.implicitly_deleted_by_id.get(&file.id()) {
                    deleted = implicit;
                    break;
                }

                if file.explicitly_deleted() {
                    deleted = true;
                    break;
                }

                file = self.find_parent(file)?;
            }

            (visited_ids, deleted)
        };

        for id in visited_ids {
            self.implicitly_deleted_by_id.insert(id, deleted);
        }

        Ok(deleted)
    }

    pub fn decrypt_key(&mut self, id: Uuid, account: &Account) -> Result<AESKey, TreeError> {
        let mut file = self.find(id)?;
        let mut visited_ids = vec![];

        loop {
            if self.key_by_id.get(&file.id()).is_some() {
                break;
            }

            if let Some(user_access) = file.user_access_keys().get(&account.username) {
                let user_access_key =
                    pubkey::get_aes_key(&account.private_key, &user_access.encrypted_by).unwrap();
                let file_key = symkey::decrypt(&user_access_key, &user_access.access_key).unwrap();
                let id = file.id();
                self.key_by_id.insert(file.id(), file_key);
                break;
            }

            visited_ids.push(file.id());
            file = self.find_parent(file)?;
        }

        for id in visited_ids.iter().rev() {
            let file = self.find(*id)?;
            let parent = self.find_parent(file)?;
            let parent_key = self.key_by_id.get(&parent.id()).unwrap();
            let encrypted_key = file.folder_access_keys();
            let decrypted_key = symkey::decrypt(&parent_key, encrypted_key).unwrap();
            self.key_by_id.insert(*id, decrypted_key);
        }

        Ok(*self.key_by_id.get(&id).unwrap())
    }

    pub fn name(&mut self, id: Uuid, account: &Account) -> Result<String, TreeError> {
        let meta = self.find(id)?;
        if let Some(name) = self.name_by_id.get(&id) {
            return Ok(name.clone());
        }

        let parent_id = meta.parent();
        let parent_key = self.decrypt_key(parent_id, account)?;

        let meta = self.find(id)?;
        let name = meta.secret_name().to_string(&parent_key).unwrap();
        self.name_by_id.insert(id, name.clone());
        Ok(name)
    }
}

impl<T: TreeLike> TreeLike for LazyTree<T> {
    type F = T::F;

    fn ids(&self) -> HashSet<Uuid> {
        self.tree.ids()
    }

    fn maybe_find(&self, id: Uuid) -> Option<&Self::F> {
        self.tree.maybe_find(id)
    }
}

#[derive(Clone)]
pub enum StagedFile<'a, Base: FileLike, Staged: FileLike> {
    Base(&'a Base),
    Staged(&'a Staged),
    Both { base: &'a Base, staged: &'a Staged },
}

impl<'a, Base: FileLike, Staged: FileLike> Display for StagedFile<'a, Base, Staged> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl<'a, Base: FileLike, Staged: FileLike> FileLike for StagedFile<'a, Base, Staged> {
    fn id(&self) -> Uuid {
        match self {
            StagedFile::Base(file) => file.id(),
            StagedFile::Staged(file) => file.id(),
            StagedFile::Both { base: _, staged: file } => file.id(),
        }
    }

    fn file_type(&self) -> FileType {
        match self {
            StagedFile::Base(file) => file.file_type(),
            StagedFile::Staged(file) => file.file_type(),
            StagedFile::Both { base: _, staged: file } => file.file_type(),
        }
    }

    fn parent(&self) -> Uuid {
        match self {
            StagedFile::Base(file) => file.parent(),
            StagedFile::Staged(file) => file.parent(),
            StagedFile::Both { base: _, staged: file } => file.parent(),
        }
    }

    fn secret_name(&self) -> &SecretFileName {
        match self {
            StagedFile::Base(file) => file.secret_name(),
            StagedFile::Staged(file) => file.secret_name(),
            StagedFile::Both { base: _, staged: file } => file.secret_name(),
        }
    }

    fn owner(&self) -> Owner {
        match self {
            StagedFile::Base(file) => file.owner(),
            StagedFile::Staged(file) => file.owner(),
            StagedFile::Both { base: _, staged: file } => file.owner(),
        }
    }

    fn explicitly_deleted(&self) -> bool {
        match self {
            StagedFile::Base(file) => file.explicitly_deleted(),
            StagedFile::Staged(file) => file.explicitly_deleted(),
            StagedFile::Both { base: _, staged: file } => file.explicitly_deleted(),
        }
    }

    fn display(&self) -> String {
        match self {
            StagedFile::Base(file) => file.display(),
            StagedFile::Staged(file) => file.display(),
            StagedFile::Both { base: _, staged: file } => file.display(),
        }
    }

    fn user_access_keys(&self) -> &HashMap<Username, UserAccessInfo> {
        match self {
            StagedFile::Base(file) => file.user_access_keys(),
            StagedFile::Staged(file) => file.user_access_keys(),
            StagedFile::Both { base: _, staged: file } => file.user_access_keys(),
        }
    }

    fn folder_access_keys(&self) -> &EncryptedFolderAccessKey {
        match self {
            StagedFile::Base(file) => file.folder_access_keys(),
            StagedFile::Staged(file) => file.folder_access_keys(),
            StagedFile::Both { base: _, staged: file } => file.folder_access_keys(),
        }
    }
}

pub struct StagedTree<'a, Base: TreeLike, Staged: TreeLike> {
    pub base: &'a Base,
    pub local: &'a Staged,
}

impl<'a, Base: TreeLike, Staged: TreeLike> TreeLike for StagedTree<'a, Base, Staged> {
    type F = StagedFile<'a, Base::F, Staged::F>;

    fn ids(&self) -> HashSet<Uuid> {
        let mut ids = self.base.ids();
        ids.extend(self.local.ids());
        ids
    }

    fn maybe_find(&self, id: Uuid) -> Option<&Self::F> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeError {
    RootNonexistent,
    FileNonexistent,
    FileParentNonexistent,
    Unexpected(String),
}
