use crate::account::Username;
use crate::file_metadata::FileType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum ShareMode {
    Write,
    Read,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Share {
    pub mode: ShareMode,
    pub shared_by: Username,
    pub shared_with: Username,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub id: Uuid,
    pub parent: Uuid,
    pub name: String,
    pub file_type: FileType,
    pub last_modified: u64,
    pub last_modified_by: Username,
    pub shares: Vec<Share>,
}

impl File {
    pub fn is_document(&self) -> bool {
        self.file_type == FileType::Document
    }

    pub fn is_folder(&self) -> bool {
        self.file_type == FileType::Folder
    }
}
