use crate::access_info::UserAccessMode;
use crate::account::Account;
use crate::crypto::{AESKey, DecryptedDocument, EncryptedDocument};
use crate::file_like::FileLike;
use crate::file_metadata::{FileType, Owner};
use crate::staged::{Stagable, StagedTree};
use crate::tree_like::{TreeLike, TreeLikeMut};
use crate::{compression_service, symkey, SharedError, SharedResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct LazyCache {
    pub name: HashMap<Uuid, String>,
    pub key: HashMap<Uuid, AESKey>,
    pub implicit_deleted: HashMap<Uuid, bool>,
    pub children: HashMap<Uuid, HashSet<Uuid>>,
}

pub trait LazyTreeLike: Sized {
    type T: Stagable;

    fn tree(&self) -> &Self::T;
    fn tree_and_cache(&mut self) -> (&Self::T, &mut LazyCache);

    fn cache(&mut self) -> &mut LazyCache {
        let (_, cache) = self.tree_and_cache();
        cache
    }

    fn access_mode(&self, owner: Owner, id: &Uuid) -> SharedResult<Option<UserAccessMode>> {
        let tree = self.tree();
        let mut file = tree.find(id)?;
        let mut max_access_mode = None;
        loop {
            let access_mode = file.access_mode(&owner);
            if access_mode > max_access_mode {
                max_access_mode = access_mode;
            }
            if file.parent() == file.id() {
                break; // root
            } else if let Some(parent) = tree.maybe_find(file.parent()) {
                file = parent
            } else {
                break; // share root
            }
        }
        Ok(max_access_mode)
    }

    fn in_pending_share(&mut self, id: &Uuid) -> SharedResult<bool> {
        let mut id = *id;
        loop {
            if self.tree().find(&id)?.parent() == self.tree().find(&id)?.id() {
                return Ok(false); // root
            } else if let Some(link) = self.link(&id)? {
                id = link;
            } else if self
                .tree()
                .maybe_find(self.tree().find(&id)?.parent())
                .is_some()
            {
                id = *self.tree().find(&id)?.parent();
            } else {
                return Ok(true); // share root
            }
        }
    }

    fn all_children(&mut self) -> SharedResult<&HashMap<Uuid, HashSet<Uuid>>> {
        let (tree, cache) = self.tree_and_cache();
        if cache.children.is_empty() {
            let mut all_children: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
            for file in tree.all_files()? {
                if !file.is_root() {
                    let mut children = all_children.remove(file.parent()).unwrap_or_default();
                    children.insert(*file.id());
                    all_children.insert(*file.parent(), children);
                }
            }
            cache.children = all_children;
        }

        Ok(&cache.children)
    }

    fn calculate_deleted(&mut self, id: &Uuid) -> SharedResult<bool> {
        let (tree, cache) = self.tree_and_cache();
        let (visited_ids, deleted) = {
            let mut file = tree.find(id)?;
            let mut visited_ids = vec![];
            let mut deleted = false;

            while !file.is_root()
                && tree.maybe_find(file.parent()).is_some()
                && !visited_ids.contains(file.parent())
            {
                visited_ids.push(*file.id());
                if let Some(&implicit) = cache.implicit_deleted.get(file.id()) {
                    deleted = implicit;
                    break;
                }

                if file.explicitly_deleted() {
                    deleted = true;
                    break;
                }

                file = match tree.maybe_find_parent(file) {
                    Some(file) => file,
                    None => {
                        if !file.user_access_keys().is_empty() {
                            break;
                        } else {
                            return Err(SharedError::FileParentNonexistent);
                        }
                    }
                }
            }

            (visited_ids, deleted)
        };

        for id in visited_ids {
            cache.implicit_deleted.insert(id, deleted);
        }

        Ok(deleted)
    }

    fn decrypt_key(&mut self, id: &Uuid, account: &Account) -> SharedResult<AESKey> {
        let (tree, cache) = self.tree_and_cache();
        let mut file_id = *tree.find(id)?.id();
        let mut visited_ids = vec![];

        loop {
            if cache.key.get(&file_id).is_some() {
                break;
            }

            let my_pk = account.public_key();

            let maybe_file_key = if let Some(user_access) = tree
                .find(&file_id)?
                .user_access_keys()
                .iter()
                .find(|access| access.encrypted_for == my_pk)
            {
                Some(user_access.decrypt(account)?)
            } else {
                None
            };
            if let Some(file_key) = maybe_file_key {
                cache.key.insert(file_id, file_key);
                break;
            }

            visited_ids.push(file_id);
            file_id = *tree.find_parent(tree.find(&file_id)?)?.id();
        }

        for id in visited_ids.iter().rev() {
            let decrypted_key = {
                let file = tree.find(id)?;
                let parent = tree.find_parent(file)?;
                let parent_key = cache.key.get(parent.id()).ok_or(SharedError::Unexpected(
                    "parent key should have been populated by prior routine",
                ))?;
                let encrypted_key = file.folder_access_key();
                symkey::decrypt(parent_key, encrypted_key)?
            };
            cache.key.insert(*id, decrypted_key);
        }

        Ok(*cache.key.get(id).ok_or(SharedError::Unexpected(
            "parent key should have been populated by prior routine (2)",
        ))?)
    }

    fn name(&mut self, id: &Uuid, account: &Account) -> SharedResult<String> {
        if let Some(name) = self.cache().name.get(id) {
            return Ok(name.clone());
        }
        let id = if let Some(link) = self.link(id)? { link } else { *id };
        let key = self.decrypt_key(&id, account)?;
        let name = self.tree().find(&id)?.secret_name().to_string(&key)?;
        self.cache().name.insert(id, name.clone());
        Ok(name)
    }

    fn link(&mut self, id: &Uuid) -> SharedResult<Option<Uuid>> {
        for link_id in self.tree().owned_ids() {
            if let FileType::Link { target } = self.tree().find(&link_id)?.file_type() {
                if id == &target && !self.calculate_deleted(&link_id)? {
                    return Ok(Some(link_id));
                }
            }
        }
        Ok(None)
    }

    /// Returns ids of files whose parent is the argument. Does not include the argument.
    /// TODO could consider returning a reference to the underlying cached value
    fn children(&mut self, id: &Uuid) -> SharedResult<HashSet<Uuid>> {
        let (tree, cache) = self.tree_and_cache();

        // Check cache
        if let Some(children) = cache.children.get(id) {
            return Ok(children.clone());
        }

        // Confirm file exists
        let file = tree.find(id)?;
        if !file.is_folder() {
            return Ok(HashSet::default());
        }

        // Populate cache
        self.all_children()?;

        // Return value from cache
        if let Some(children) = self.cache().children.get(id) {
            return Ok(children.clone());
        }

        Ok(HashSet::new())
    }

    // todo: cache?
    fn children_using_links(&mut self, id: &Uuid) -> SharedResult<HashSet<Uuid>> {
        let tree = self.tree();
        let id = match tree.find(id)?.file_type() {
            FileType::Document | FileType::Folder => *id,
            FileType::Link { target } => target,
        };
        self.children(&id)
    }

    /// Returns ids of files for which the argument is an ancestor—the files' children, recursively. Does not include the argument.
    /// This function tolerates cycles.
    fn descendants(&mut self, id: &Uuid) -> SharedResult<HashSet<Uuid>> {
        // todo: caching?
        let mut result = HashSet::new();
        let mut to_process = vec![*id];
        let mut i = 0;
        while i < to_process.len() {
            let new_descendents = self
                .children(&to_process[i])?
                .into_iter()
                .filter(|f| !result.contains(f))
                .collect::<Vec<Uuid>>();
            // TODO could consider optimizing by not exploring documents
            to_process.extend(new_descendents.iter());
            result.extend(new_descendents.into_iter());
            i += 1;
        }
        Ok(result)
    }

    fn descendants_using_links(&mut self, id: &Uuid) -> SharedResult<HashSet<Uuid>> {
        // todo: caching?
        let mut result = HashSet::new();
        let mut to_process = vec![*id];
        let mut i = 0;
        while i < to_process.len() {
            let new_descendents = self
                .children_using_links(&to_process[i])?
                .into_iter()
                .filter(|f| !result.contains(f))
                .collect::<Vec<Uuid>>();
            // TODO could consider optimizing by not exploring documents
            to_process.extend(new_descendents.iter());
            result.extend(new_descendents.into_iter());
            i += 1;
        }
        Ok(result)
    }

    // todo: move to TreeLike
    /// Returns ids of files for which the argument is a descendent—the files' parent, recursively. Does not include the argument.
    /// This function tolerates cycles.
    fn ancestors(&self, id: &Uuid) -> SharedResult<HashSet<Uuid>> {
        let tree = self.tree();
        let mut result = HashSet::new();
        let mut current_file = tree.find(id)?;
        while !current_file.is_root()
            && tree.maybe_find(current_file.parent()).is_some()
            && !result.contains(current_file.parent())
        {
            result.insert(*current_file.parent());
            current_file = tree.find_parent(current_file)?;
        }
        Ok(result)
    }

    fn decrypt_document(
        &mut self, id: &Uuid, encrypted: &EncryptedDocument, account: &Account,
    ) -> SharedResult<DecryptedDocument> {
        let key = self.decrypt_key(id, account)?;
        let compressed = symkey::decrypt(&key, encrypted)?;
        compression_service::decompress(&compressed)
    }

    // todo: optimize
    fn assert_names_decryptable(&mut self, account: &Account) -> SharedResult<()> {
        let tree = self.tree();
        for id in tree.owned_ids() {
            if self.name(&id, account).is_err() {
                return Err(SharedError::ValidationFailure(
                    ValidationFailure::NonDecryptableFileName(id),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LazyTreeRef<'t, T: Stagable> {
    pub tree: &'t T,
    pub cache: LazyCache,
}

impl<'t, T: Stagable> LazyTreeRef<'t, T> {
    pub fn new(tree: &'t T) -> Self {
        Self { tree, cache: Default::default() }
    }
}

impl<'t, T: Stagable> LazyTreeLike for LazyTreeRef<'t, T> {
    type T = T;

    fn tree(&self) -> &Self::T {
        self.tree
    }

    fn tree_and_cache(&mut self) -> (&Self::T, &mut LazyCache) {
        (self.tree, &mut self.cache)
    }
}

#[derive(Debug)]
pub struct LazyTree<T: Stagable> {
    pub tree: T,
    pub cache: LazyCache,
}

impl<T: Stagable> LazyTree<T> {
    pub fn new(tree: T) -> Self {
        Self { tree, cache: Default::default() }
    }
}

impl<T: Stagable> LazyTreeLike for LazyTree<T> {
    type T = T;

    fn tree(&self) -> &Self::T {
        &self.tree
    }

    fn tree_and_cache(&mut self) -> (&Self::T, &mut LazyCache) {
        (&self.tree, &mut self.cache)
    }
}

impl<T: Stagable> LazyTree<T> {
    pub fn stage<S: Stagable<F = T::F>>(self, staged: &mut S) -> LazyStaged1<T, S> {
        // todo: optimize by performing minimal updates on self caches
        LazyTree::<StagedTree<T, S>> {
            cache: LazyCache {
                name: HashMap::new(),
                key: self.cache.key,
                implicit_deleted: HashMap::new(),
                children: HashMap::new(),
            },
            tree: StagedTree::new(self.tree, staged),
        }
    }
}

pub type Staged1<'s2, S1, S2> = StagedTree<'s2, S1, S2>;
pub type LazyStaged1<'s2, S1, S2> = LazyTree<Staged1<'s2, S1, S2>>;
pub type Staged2<'s2, 's3, S1, S2, S3> = StagedTree<'s3, Staged1<'s2, S1, S2>, S3>;
pub type LazyStaged2<'s2, 's3, S1, S2, S3> = LazyTree<Staged2<'s2, 's3, S1, S2, S3>>;
pub type Staged3<'s2, 's3, 's4, S1, S2, S3, S4> =
    StagedTree<'s4, Staged2<'s2, 's3, S1, S2, S3>, S4>;
pub type LazyStaged3<'s2, 's3, 's4, S1, S2, S3, S4> =
    LazyTree<Staged3<'s2, 's3, 's4, S1, S2, S3, S4>>;
pub type Staged4<'s2, 's3, 's4, 's5, S1, S2, S3, S4, S5> =
    StagedTree<'s5, Staged3<'s2, 's3, 's4, S1, S2, S3, S4>, S5>;
pub type LazyStaged4<'s2, 's3, 's4, 's5, S1, S2, S3, S4, S5> =
    LazyTree<Staged4<'s2, 's3, 's4, 's5, S1, S2, S3, S4, S5>>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub enum ValidationFailure {
    Orphan(Uuid),
    Cycle(HashSet<Uuid>),
    PathConflict(HashSet<Uuid>),
    NonFolderWithChildren(Uuid),
    FileWithDifferentOwnerParent(Uuid),
    NonDecryptableFileName(Uuid),
    SharedLink { link: Uuid, shared_ancestor: Uuid },
    DuplicateLink { target: Uuid },
    BrokenLink(Uuid),
    OwnedLink(Uuid),
}

impl<'s, Base, Staged> LazyStaged1<'s, Base, Staged>
where
    Base: Stagable,
    Staged: Stagable<F = Base::F>,
{
    // todo: incrementalism
    pub fn promote(self) -> LazyTree<Base> {
        let staged = self.tree.staged;
        let mut base = self.tree.base;
        for id in staged.owned_ids() {
            if let Some(removed) = staged.remove(id) {
                base.insert(removed);
            }
        }

        LazyTree {
            tree: base,
            cache: LazyCache {
                name: HashMap::new(),
                key: HashMap::new(),
                implicit_deleted: HashMap::new(),
                children: HashMap::new(),
            },
        }
    }

    // todo: incrementalism
    pub fn unstage(self) -> (LazyTree<Base>, &'s mut Staged) {
        (
            LazyTree {
                tree: self.tree.base,
                cache: LazyCache {
                    name: HashMap::new(),
                    key: HashMap::new(),
                    implicit_deleted: HashMap::new(),
                    children: HashMap::new(),
                },
            },
            self.tree.staged,
        )
    }
}

impl<T: Stagable> TreeLike for LazyTree<T> {
    type F = T::F;

    fn ids(&self) -> HashSet<&Uuid> {
        self.tree.ids()
    }

    fn maybe_find(&self, id: &Uuid) -> Option<&Self::F> {
        self.tree.maybe_find(id)
    }
}

impl<T: Stagable> TreeLikeMut for LazyTree<T> {
    fn insert(&mut self, f: Self::F) -> Option<Self::F> {
        self.tree.insert(f)
    }

    fn remove(&mut self, id: Uuid) -> Option<Self::F> {
        self.tree.remove(id)
    }
}
