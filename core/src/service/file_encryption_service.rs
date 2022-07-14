use libsecp256k1::PublicKey;
use std::collections::HashMap;

use uuid::Uuid;

use lockbook_shared::account::Account;
use lockbook_shared::crypto::*;
use lockbook_shared::file_metadata::{
    CoreFile, DecryptedFiles, EncryptedFiles, FileType, Owner, UnsignedFile,
};
use lockbook_shared::tree::{FileLike, FileMetaMapExt};
use lockbook_shared::{pubkey, symkey};

use crate::model::errors::{core_err_unexpected, CoreError};

pub fn create_root(account: &Account) -> Result<UnsignedFile, CoreError> {}

/// Converts a DecryptedFileMetadata to a FileMetadata using its decrypted parent key. Sharing is
/// not supported; user access keys are encrypted for the provided account. This is a pure function.
pub fn encrypt_metadatum(
    account: &Account, public_key: &PublicKey, parent_key: &AESKey, target: &CoreFile,
) -> Result<UnsignedFile, CoreError> {
    let user_access_keys = if target.is_root() {
        encrypt_user_access_keys(account, public_key, &target.decrypted_access_key)?
    } else {
        Default::default()
    };
    Ok(UnsignedFile {
        id: target.id,
        file_type: target.file_type,
        parent: target.parent,
        name: encrypt_file_name(&target.decrypted_name, parent_key)?,
        owner: target.owner.clone(),
        is_deleted: target.deleted,
        user_access_keys,
        folder_access_keys: encrypt_folder_access_keys(&target.decrypted_access_key, parent_key)?,
    })
}

/// Converts a set of DecryptedFileMetadata's to EncryptedFileMetadata's. All parents of files must be
/// included in files. Sharing is not supported; user access keys are encrypted for the provided
/// account. This is a pure function.
/// This is O(n) now with hashmaps
pub fn encrypt_metadata(
    account: &Account, public_key: &PublicKey, files: &DecryptedFiles,
) -> Result<EncryptedFiles, CoreError> {
    let mut result = HashMap::new();
    for target in files.values() {
        let parent_key = files.maybe_find(target.parent).ok_or_else(|| {
                CoreError::Unexpected(String::from(
                    "parent metadata missing during call to file_encrpytion_service::encrypt_metadata",
                ))
            })?
            .decrypted_access_key;
        result.push(encrypt_metadatum(account, public_key, &parent_key, target)?);
    }
    Ok(result)
}

fn encrypt_file_name(
    decrypted_name: &str, parent_key: &AESKey,
) -> Result<SecretFileName, CoreError> {
    symkey::encrypt_and_hmac(parent_key, decrypted_name).map_err(core_err_unexpected)
}

fn encrypt_user_access_keys(
    account: &Account, decrypted_file_key: &AESKey,
) -> Result<HashMap<String, UserAccessInfo>, CoreError> {
    let user_key =
        pubkey::get_aes_key(&account.private_key, public_key).map_err(core_err_unexpected)?;
    let encrypted_file_key =
        symkey::encrypt(&user_key, decrypted_file_key).map_err(core_err_unexpected)?;
    let mut result = HashMap::new();
    result.insert(
        account.username.clone(),
        UserAccessInfo {
            username: account.username.clone(),
            encrypted_by: account.public_key(),
            access_key: encrypted_file_key,
        },
    );
    Ok(result)
}

fn encrypt_folder_access_keys(
    target_key: &AESKey, parent_key: &AESKey,
) -> Result<EncryptedFolderAccessKey, CoreError> {
    symkey::encrypt(parent_key, target_key).map_err(core_err_unexpected)
}

/// Converts a FileMetadata to a DecryptedFileMetadata using its decrypted parent key. Sharing is
/// not supported; user access keys not for the provided account are ignored. This is a pure function.
pub fn decrypt_metadatum(
    parent_key: &AESKey, target: &UnsignedFile,
) -> Result<CoreFile, CoreError> {
    Ok(CoreFile {
        id: target.id,
        file_type: target.file_type,
        parent: target.parent,
        decrypted_name: decrypt_file_name(&target.name, parent_key)?,
        owner: target.owner.clone(),
        metadata_version: target.metadata_version,
        content_version: target.content_version,
        deleted: target.is_deleted,
        decrypted_access_key: decrypt_folder_access_keys(&target.folder_access_keys, parent_key)?,
    })
}

/// Converts a set of FileMetadata's to DecryptedFileMetadata's. All parents of files must be
/// included in files. Sharing is not supported; user access keys not for the provided account are
/// ignored. This is a pure function.
pub fn decrypt_metadata(
    account: &Account, files: &EncryptedFiles, key_cache: &mut HashMap<Uuid, AESKey>,
) -> Result<DecryptedFiles, CoreError> {
    let mut result = HashMap::new();

    for target in files.values() {
        let parent_key = decrypt_file_key(account, target.parent, files, key_cache)?;
        let decrypted_metadatum = decrypt_metadatum(&parent_key, target)?;
        result.push(decrypted_metadatum);
    }
    Ok(result)
}

/// Decrypts the file key given a target and its ancestors. All ancestors of target, as well as
/// target itself, must be included in target_with_ancestors.
fn decrypt_file_key(
    account: &Account, target_id: Uuid, target_with_ancestors: &EncryptedFiles,
    key_cache: &mut HashMap<Uuid, AESKey>,
) -> Result<AESKey, CoreError> {
    if let Some(key) = key_cache.get(&target_id) {
        return Ok(*key);
    }

    let target = target_with_ancestors.maybe_find(target_id).ok_or_else(|| {
        CoreError::Unexpected(String::from(
            "target or ancestor missing during call to file_encryption_service::decrypt_file_key",
        ))
    })?;

    let key = match target.user_access_keys.get(&account.username) {
        Some(user_access) => {
            let user_access_key =
                pubkey::get_aes_key(&account.private_key, &user_access.encrypted_by)
                    .map_err(core_err_unexpected)?;
            symkey::decrypt(&user_access_key, &user_access.access_key)
                .map_err(core_err_unexpected)?
        }
        None => {
            let parent_key =
                decrypt_file_key(account, target.parent, target_with_ancestors, key_cache)?;
            symkey::decrypt(&parent_key, &target.folder_access_keys).map_err(core_err_unexpected)?
        }
    };

    key_cache.insert(target_id, key);

    Ok(key)
}

fn decrypt_file_name(
    encrypted_name: &SecretFileName, parent_key: &AESKey,
) -> Result<String, CoreError> {
    symkey::decrypt_and_verify(parent_key, encrypted_name).map_err(core_err_unexpected)
}

fn decrypt_folder_access_keys(
    encrypted_keys: &EncryptedFolderAccessKey, parent_key: &AESKey,
) -> Result<AESKey, CoreError> {
    symkey::decrypt(parent_key, encrypted_keys).map_err(core_err_unexpected)
}

pub fn encrypt_document(
    document: &[u8], metadata: &CoreFile,
) -> Result<EncryptedDocument, CoreError> {
    symkey::encrypt(&metadata.decrypted_access_key, &document.to_vec()).map_err(core_err_unexpected)
}

pub fn decrypt_document(
    document: &EncryptedDocument, metadata: &CoreFile,
) -> Result<DecryptedDocument, CoreError> {
    symkey::decrypt(&metadata.decrypted_access_key, document).map_err(core_err_unexpected)
}
