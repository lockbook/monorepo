use serde::{Deserialize, Serialize};
use std::clone::Clone;
use uuid::Uuid;

use crate::model::crypto::{FolderAccessInfo, UserAccessInfo};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum FileType {
    Document,
    Folder,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ClientFileMetadata {
    /// Is this a file or a folder?
    pub file_type: FileType,

    /// Immutable unique identifier for everything related to this file
    pub id: Uuid,

    /// Human readable name for this file. Does not need to be unique TODO make this encrypted / hashed / etc.
    pub name: String,

    /// Where this file lives relative to your other files
    pub parent_id: Uuid,

    /// DB generated timestamp representing the last time the content of a file was updated
    pub content_version: u64,

    /// DB generated timestamp representing the last time the metadata for this file changed
    pub metadata_version: u64,

    /// True if this is a new file, that has never been synced before
    pub new: bool,

    /// True if there are changes to content that need to be synced
    pub document_edited: bool,

    /// True if there are changes to metadata that need to be synced
    pub metadata_changed: bool,

    /// True if the user attempted to delete this file locally. Once the server also deletes this file, the content and the associated metadata are deleted locally.
    pub deleted: bool,

    pub user_access_keys: HashMap<String, UserAccessInfo>,
    pub folder_access_keys: FolderAccessInfo,
}
