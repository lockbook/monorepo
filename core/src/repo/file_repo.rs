use crate::CoreError;
use crate::model::repo::RepoSource;
use crate::model::state::Config;
use crate::repo::account_repo;
use crate::repo::digest_repo;
use crate::repo::document_repo;
use crate::repo::metadata_repo;
use crate::service::file_compression_service;
use crate::service::file_encryption_service;
use crate::utils;
use lockbook_models::crypto::DecryptedDocument;
use lockbook_models::crypto::EncryptedDocument;
use lockbook_models::file_metadata::DecryptedFileMetadata;
use lockbook_models::file_metadata::FileMetadata;
use lockbook_models::file_metadata::FileMetadataDiff;
use lockbook_models::file_metadata::FileType;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashSet;
use uuid::Uuid;

pub fn get_all_metadata_changes(config: &Config) -> Result<Vec<FileMetadataDiff>, CoreError> {
    let local = metadata_repo::get_all(config, RepoSource::Local)?;
    let base = metadata_repo::get_all(config, RepoSource::Base)?;

    let new = local
        .iter()
        .filter(|l| !base.iter().any(|r| r.id == l.id))
        .map(|l| FileMetadataDiff::new(l));
    let changed = local
        .iter()
        .filter_map(|l| match base.iter().find(|r| r.id == l.id) {
            Some(r) => Some((l, r)),
            None => None,
        })
        .map(|(l, r)| FileMetadataDiff::new_diff(r.parent, &r.name, l));

    Ok(new.chain(changed).collect())
}

pub fn get_all_with_document_changes(config: &Config) -> Result<Vec<Uuid>, CoreError> {
    let all = get_all_metadata(config, RepoSource::Local)?;
    let not_deleted = utils::filter_not_deleted(&all);
    let not_deleted_with_document_changes = not_deleted
        .into_iter()
        .map(|f| document_repo::maybe_get(config, RepoSource::Local, f.id).map(|r| r.map(|_| f.id)))
        .collect::<Result<Vec<Option<Uuid>>, CoreError>>()?
        .into_iter()
        .filter_map(|id| id)
        .collect();
    Ok(not_deleted_with_document_changes)
}

/// Adds or updates the metadata of a file on disk.
/// Disk optimization opportunity: this function needlessly writes to disk when setting local metadata = base metadata.
/// CPU optimization opportunity: this function needlessly decrypts all metadata rather than just ancestors of metadata parameter.
pub fn insert_metadata(
    config: &Config,
    source: RepoSource,
    metadata: &DecryptedFileMetadata,
) -> Result<(), CoreError> {
    // encrypt metadata
    let account = account_repo::get(config)?;
    let all_metadata = get_all_metadata(config, source)?;
    let all_metadata_with_this_change_staged = utils::stage(&all_metadata, &[metadata.clone()])
        .into_iter()
        .map(|(f, _)| f)
        .collect::<Vec<DecryptedFileMetadata>>();
    let parent = utils::find_parent(&all_metadata_with_this_change_staged, metadata.id)?;
    let encrypted_metadata = file_encryption_service::encrypt_metadatum(
        &account,
        &parent.decrypted_access_key,
        metadata,
    )?;

    // perform insertion
    metadata_repo::insert(config, source, &encrypted_metadata)?;

    // remove local if local == base
    if let Some(opposite) =
        metadata_repo::maybe_get(config, source.opposite(), encrypted_metadata.id)?
    {
        if utils::slices_equal(&opposite.name.hmac, &encrypted_metadata.name.hmac)
            && opposite.parent == metadata.parent
            && opposite.deleted == metadata.deleted
        {
            metadata_repo::delete(config, RepoSource::Local, metadata.id)?;
        }
    }

    Ok(())
}

pub fn get_metadata(
    config: &Config,
    source: RepoSource,
    id: Uuid,
) -> Result<DecryptedFileMetadata, CoreError> {
    maybe_get_metadata(config, source, id).and_then(|f| f.ok_or(CoreError::FileNonexistent))
}

pub fn maybe_get_metadata(
    config: &Config,
    source: RepoSource,
    id: Uuid,
) -> Result<Option<DecryptedFileMetadata>, CoreError> {
    let all_metadata = get_all_metadata(config, source)?;
    Ok(utils::maybe_find(&all_metadata, id))
}

pub fn get_all_metadata(
    config: &Config,
    source: RepoSource,
) -> Result<Vec<DecryptedFileMetadata>, CoreError> {
    let account = account_repo::get(config)?;
    let base = metadata_repo::get_all(config, RepoSource::Base)?;
    match source {
        RepoSource::Local => {
            let local = metadata_repo::get_all(config, RepoSource::Local)?;
            let staged = utils::stage_encrypted(&base, &local)
                .into_iter()
                .map(|(f, _)| f)
                .collect::<Vec<FileMetadata>>();
            file_encryption_service::decrypt_metadata(&account, &staged)
        }
        RepoSource::Base => file_encryption_service::decrypt_metadata(&account, &base),
    }
}

pub fn get_all_metadata_with_encrypted_changes(
    config: &Config,
    source: RepoSource,
    changes: &[FileMetadata],
) -> Result<Vec<DecryptedFileMetadata>, CoreError> {
    let account = account_repo::get(config)?;
    let base = metadata_repo::get_all(config, RepoSource::Base)?;
    let sourced = match source {
        RepoSource::Local => {
            let local = metadata_repo::get_all(config, RepoSource::Local)?;
            utils::stage_encrypted(&base, &local)
                .into_iter()
                .map(|(f, _)| f)
                .collect()
        }
        RepoSource::Base => base,
    };
    let staged = utils::stage_encrypted(&sourced, &changes)
        .into_iter()
        .map(|(f, _)| f)
        .collect::<Vec<FileMetadata>>();
    file_encryption_service::decrypt_metadata(&account, &staged)
}

/// Adds or updates the content of a document on disk.
/// Disk optimization opportunity: this function needlessly writes to disk when setting local content = base content.
/// CPU optimization opportunity: this function needlessly decrypts all metadata rather than just ancestors of metadata parameter.
pub fn insert_document(
    config: &Config,
    source: RepoSource,
    metadata: &DecryptedFileMetadata,
    document: &[u8],
) -> Result<(), CoreError> {
    // encrypt document and compute digest
    let digest = Sha256::digest(&document);
    let compressed_document = file_compression_service::compress(&document)?;
    let encrypted_document =
        file_encryption_service::encrypt_document(&compressed_document, &metadata)?;

    // perform insertions
    document_repo::insert(config, source, metadata.id, &encrypted_document)?;
    digest_repo::insert(config, source, metadata.id, &digest)?;

    // remove local if local == base
    if let Some(opposite) = digest_repo::maybe_get(config, source.opposite(), metadata.id)? {
        if utils::slices_equal(&opposite, &digest) {
            document_repo::delete(config, RepoSource::Local, metadata.id)?;
            digest_repo::delete(config, RepoSource::Local, metadata.id)?;
        }
    }

    Ok(())
}

pub fn get_document(
    config: &Config,
    source: RepoSource,
    id: Uuid,
) -> Result<DecryptedDocument, CoreError> {
    maybe_get_document(config, source, id).and_then(|f| f.ok_or(CoreError::FileNonexistent))
}

pub fn maybe_get_document(
    config: &Config,
    source: RepoSource,
    id: Uuid,
) -> Result<Option<DecryptedDocument>, CoreError> {
    let maybe_metadata = maybe_get_metadata(config, source, id)?;
    let maybe_encrypted = match source {
        RepoSource::Local => match document_repo::maybe_get(config, RepoSource::Local, id)? {
            Some(metadata) => Some(metadata),
            None => document_repo::maybe_get(config, RepoSource::Base, id)?,
        },
        RepoSource::Base => document_repo::maybe_get(config, RepoSource::Base, id)?,
    };
    let maybe_compressed = match (maybe_metadata, maybe_encrypted) {
        (Some(metadata), Some(encrypted)) => Some(file_encryption_service::decrypt_document(
            &encrypted, &metadata,
        )?),
        _ => None,
    };
    match maybe_compressed {
        Some(compressed) => Ok(Some(file_compression_service::decompress(&compressed)?)),
        None => Ok(None),
    }
}

/// Updates base files to match local files.
pub fn promote(config: &Config) -> Result<(), CoreError> {
    let base_metadata = metadata_repo::get_all(config, RepoSource::Base)?;
    let local_metadata = metadata_repo::get_all(config, RepoSource::Local)?;
    let staged_metadata = utils::stage_encrypted(&base_metadata, &local_metadata);
    let staged_everything = staged_metadata
        .into_iter()
        .map(|(f, _)| {
            Ok((
                f.clone(),
                match document_repo::maybe_get(config, RepoSource::Local, f.id)? {
                    Some(document) => Some(document),
                    None => document_repo::maybe_get(config, RepoSource::Base, f.id)?,
                },
                match digest_repo::maybe_get(config, RepoSource::Local, f.id)? {
                    Some(digest) => Some(digest),
                    None => digest_repo::maybe_get(config, RepoSource::Base, f.id)?,
                },
            ))
        })
        .collect::<Result<Vec<(FileMetadata, Option<EncryptedDocument>, Option<Vec<u8>>)>, CoreError>>()?;

    metadata_repo::delete_all(config, RepoSource::Base)?;
    document_repo::delete_all(config, RepoSource::Base)?;
    digest_repo::delete_all(config, RepoSource::Base)?;

    for (metadata, maybe_document, maybe_digest) in staged_everything {
        metadata_repo::insert(config, RepoSource::Base, &metadata)?;
        if let Some(document) = maybe_document {
            document_repo::insert(config, RepoSource::Base, metadata.id, &document)?;
        }
        if let Some(digest) = maybe_digest {
            digest_repo::insert(config, RepoSource::Base, metadata.id, &digest)?;
        }
    }

    metadata_repo::delete_all(config, RepoSource::Local)?;
    document_repo::delete_all(config, RepoSource::Local)?;
    digest_repo::delete_all(config, RepoSource::Local)
}

/// Removes deleted files which are safe to delete. Call this function after a set of operations rather than in-between
/// each operation because otherwise you'll prune e.g. a file that was moved out of a folder that was deleted.
pub fn prune_deleted(config: &Config) -> Result<(), CoreError> {
    // If a file is deleted or has a deleted ancestor, we say that it is deleted. Whether a file is deleted is specific
    // to the source (base or local). We cannot prune (delete from disk) a file in one source and not in the other in
    // order to preserve the semantics of having a file present on one, the other, or both (unmodified/new/modified).
    // For a file to be pruned, it must be deleted on both sources but also have no non-deleted descendants on either
    // source - otherwise, the metadata for those descendants can no longer be decrypted. For an example of a situation
    // where this is important, see the test prune_deleted_document_moved_from_deleted_folder_local_only.

    // find files deleted on base and local
    let all_base_metadata = get_all_metadata(config, RepoSource::Base)?;
    let deleted_base_metadata = utils::filter_deleted(&all_base_metadata);
    let all_local_metadata = get_all_metadata(config, RepoSource::Local)?;
    let deleted_local_metadata = utils::filter_deleted(&all_local_metadata);
    let deleted_both_metadata = deleted_base_metadata
        .into_iter()
        .filter(|f| utils::maybe_find(&deleted_local_metadata, f.id).is_some())
        .collect::<Vec<DecryptedFileMetadata>>();

    // exclude files with not deleted descendants i.e. exclude files that are the ancestors of not deleted files
    let all_ids = all_base_metadata
        .iter()
        .chain(all_local_metadata.iter())
        .map(|f| f.id)
        .collect::<HashSet<Uuid>>();
    let not_deleted_either_ids = all_ids
        .into_iter()
        .filter(|&id| utils::maybe_find(&deleted_both_metadata, id).is_none())
        .collect::<HashSet<Uuid>>();
    let ancestors_of_not_deleted_base_ids = not_deleted_either_ids
        .iter()
        .flat_map(|&id| utils::find_ancestors(&all_base_metadata, id))
        .map(|f| f.id)
        .collect::<HashSet<Uuid>>();
    let ancestors_of_not_deleted_local_ids = not_deleted_either_ids
        .iter()
        .flat_map(|&id| utils::find_ancestors(&all_local_metadata, id))
        .map(|f| f.id)
        .collect::<HashSet<Uuid>>();
    let deleted_both_without_deleted_descendants_ids =
        deleted_both_metadata.into_iter().filter(|f| {
            !ancestors_of_not_deleted_base_ids.contains(&f.id)
                && !ancestors_of_not_deleted_local_ids.contains(&f.id)
        });

    // remove files from disk
    for file in deleted_both_without_deleted_descendants_ids {
        delete_metadata(config, file.id)?;
        if file.file_type == FileType::Document {
            delete_document(config, file.id)?;
        }
    }
    Ok(())
}

fn delete_metadata(config: &Config, id: Uuid) -> Result<(), CoreError> {
    metadata_repo::delete(config, RepoSource::Local, id)?;
    metadata_repo::delete(config, RepoSource::Base, id)
}

fn delete_document(config: &Config, id: Uuid) -> Result<(), CoreError> {
    document_repo::delete(config, RepoSource::Local, id)?;
    document_repo::delete(config, RepoSource::Base, id)?;
    digest_repo::delete(config, RepoSource::Local, id)?;
    digest_repo::delete(config, RepoSource::Base, id)
}

#[cfg(test)]
mod unit_tests {
    use lockbook_models::file_metadata::FileType;
    use uuid::Uuid;

    use crate::model::repo::RepoSource;
    use crate::model::state::temp_config;
    use crate::repo::{account_repo, file_repo};
    use crate::service::{file_service, test_utils};

    macro_rules! assert_metadata_changes_count (
        ($db:expr, $total:literal) => {
            assert_eq!(
                file_repo::get_all_metadata_changes($db)
                    .unwrap()
                    .len(),
                $total
            );
        }
    );

    macro_rules! assert_document_changes_count (
        ($db:expr, $total:literal) => {
            assert_eq!(
                file_repo::get_all_with_document_changes($db)
                    .unwrap()
                    .len(),
                $total
            );
        }
    );

    macro_rules! assert_metadata_nonexistent (
        ($db:expr, $source:expr, $id:expr) => {
            assert_eq!(
                file_repo::maybe_get_metadata($db, $source, $id).unwrap(),
                None,
            );
        }
    );

    macro_rules! assert_metadata_eq (
        ($db:expr, $source:expr, $id:expr, $metadata:expr) => {
            assert_eq!(
                file_repo::maybe_get_metadata($db, $source, $id).unwrap(),
                Some($metadata.clone()),
            );
        }
    );

    macro_rules! assert_document_eq (
        ($db:expr, $source:expr, $id:expr, $document:literal) => {
            assert_eq!(
                file_repo::maybe_get_document($db, $source, $id).unwrap(),
                Some($document.to_vec()),
            );
        }
    );

    macro_rules! assert_metadata_count (
        ($db:expr, $source:expr, $total:literal) => {
            assert_eq!(
                file_repo::get_all_metadata($db, $source)
                    .unwrap()
                    .len(),
                $total
            );
        }
    );

    macro_rules! assert_document_count (
        ($db:expr, $source:expr, $total:literal) => {
            assert_eq!(
                file_repo::get_all_metadata($db, $source)
                    .unwrap()
                    .iter()
                    .filter(|&f| file_repo::maybe_get_document($db, $source, f.id).unwrap().is_some())
                    .count(),
                $total
            );
        }
    );

    #[test]
    fn insert_metadata() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();

        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn get_metadata() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        let result = file_repo::get_metadata(config, RepoSource::Local, root.id).unwrap();

        assert_eq!(result, root);
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn get_metadata_nonexistent() {
        let config = &temp_config();
        let account = test_utils::generate_account();

        account_repo::insert(config, &account).unwrap();
        let result = file_repo::get_metadata(config, RepoSource::Local, Uuid::new_v4());

        assert!(result.is_err());
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 0);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn get_metadata_local_falls_back_to_base() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        let result = file_repo::get_metadata(config, RepoSource::Local, root.id).unwrap();

        assert_eq!(result, root);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn get_metadata_local_prefers_local() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let mut root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();

        root.decrypted_name += " 2";

        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        let result = file_repo::get_metadata(config, RepoSource::Local, root.id).unwrap();

        assert_eq!(result, root);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn maybe_get_metadata() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        let result = file_repo::maybe_get_metadata(config, RepoSource::Local, root.id).unwrap();

        assert_eq!(result, Some(root));
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn maybe_get_metadata_nonexistent() {
        let config = &temp_config();
        let account = test_utils::generate_account();

        account_repo::insert(config, &account).unwrap();
        let result =
            file_repo::maybe_get_metadata(config, RepoSource::Local, Uuid::new_v4()).unwrap();

        assert!(result.is_none());
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 0);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn insert_document() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();

        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn get_document() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();
        let result = file_repo::get_document(config, RepoSource::Local, document.id).unwrap();

        assert_eq!(result, b"document content");
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn get_document_nonexistent() {
        let config = &temp_config();
        let account = test_utils::generate_account();

        account_repo::insert(config, &account).unwrap();
        let result = file_repo::get_document(config, RepoSource::Local, Uuid::new_v4());

        assert!(result.is_err());
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 0);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn get_document_local_falls_back_to_base() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();
        let result = file_repo::get_document(config, RepoSource::Local, document.id).unwrap();

        assert_eq!(result, b"document content");
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn get_document_local_prefers_local() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content 2")
            .unwrap();
        let result = file_repo::get_document(config, RepoSource::Local, document.id).unwrap();

        assert_eq!(result, b"document content 2");
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn maybe_get_document() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();
        let result = file_repo::maybe_get_document(config, RepoSource::Local, document.id).unwrap();

        assert_eq!(result, Some(b"document content".to_vec()));
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn maybe_get_document_nonexistent() {
        let config = &temp_config();
        let account = test_utils::generate_account();

        account_repo::insert(config, &account).unwrap();
        let result =
            file_repo::maybe_get_document(config, RepoSource::Local, Uuid::new_v4()).unwrap();

        assert!(result.is_none());
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 0);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn no_changes() {
        let config = &temp_config();
        let account = test_utils::generate_account();

        account_repo::insert(config, &account).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 0);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn new() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert!(file_repo::get_all_metadata_changes(config).unwrap()[0]
            .old_parent_and_name
            .is_none());
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn new_idempotent() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert!(file_repo::get_all_metadata_changes(config).unwrap()[0]
            .old_parent_and_name
            .is_none());
        assert_metadata_count!(config, RepoSource::Base, 0);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn matching_base_and_local() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn matching_local_and_base() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn move_unmove() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let folder = file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        document.parent = folder.id;
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert!(file_repo::get_all_metadata_changes(config).unwrap()[0]
            .old_parent_and_name
            .is_some());
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        document.parent = root.id;
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn rename_unrename() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let folder = file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        document.decrypted_name = String::from("document 2");
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert!(file_repo::get_all_metadata_changes(config).unwrap()[0]
            .old_parent_and_name
            .is_some());
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        document.decrypted_name = String::from("document");
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn delete() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let folder = file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        document.deleted = true;
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert!(file_repo::get_all_metadata_changes(config).unwrap()[0]
            .old_parent_and_name
            .is_some());
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn multiple_metadata_edits() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let mut root = file_service::create_root(&account.username);
        let mut folder =
            file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        root.decrypted_name = String::from("root 2");
        folder.deleted = true;
        document.parent = folder.id;
        let document2 =
            file_service::create(FileType::Document, root.id, "document 2", &account.username);
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document2).unwrap();

        assert_metadata_changes_count!(config, 4);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 4);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn document_edit() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 1);
        assert_eq!(
            file_repo::get_all_with_document_changes(config).unwrap()[0],
            document.id
        );
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn document_edit_idempotent() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 1);
        assert_eq!(
            file_repo::get_all_with_document_changes(config).unwrap()[0],
            document.id
        );
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn document_edit_revert() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        file_repo::insert_document(config, RepoSource::Local, &document, b"document content 2")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 1);
        assert_eq!(
            file_repo::get_all_with_document_changes(config).unwrap()[0],
            document.id
        );
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        file_repo::insert_document(config, RepoSource::Local, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn document_edit_manual_promote() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        file_repo::insert_document(config, RepoSource::Local, &document, b"document content 2")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 1);
        assert_eq!(
            file_repo::get_all_with_document_changes(config).unwrap()[0],
            document.id
        );
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        file_repo::insert_document(config, RepoSource::Base, &document, b"document content 2")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn promote() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let mut root = file_service::create_root(&account.username);
        let mut folder =
            file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let mut document =
            file_service::create(FileType::Document, folder.id, "document", &account.username);
        let document2 = file_service::create(
            FileType::Document,
            folder.id,
            "document 2",
            &account.username,
        );

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document2).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document2, b"document 2 content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 4);
        assert_metadata_count!(config, RepoSource::Local, 4);
        assert_document_count!(config, RepoSource::Base, 2);
        assert_document_count!(config, RepoSource::Local, 2);

        root.decrypted_name = String::from("root 2");
        folder.deleted = true;
        document.parent = root.id;
        let document3 =
            file_service::create(FileType::Document, root.id, "document 3", &account.username);
        file_repo::insert_metadata(config, RepoSource::Local, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document3).unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content 2")
            .unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document3, b"document 3 content")
            .unwrap();

        assert_metadata_changes_count!(config, 4);
        assert_document_changes_count!(config, 2);
        assert_metadata_count!(config, RepoSource::Base, 4);
        assert_metadata_count!(config, RepoSource::Local, 5);
        assert_document_count!(config, RepoSource::Base, 2);
        assert_document_count!(config, RepoSource::Local, 3);

        file_repo::promote(config).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_eq!(config, RepoSource::Base, root.id, root);
        assert_metadata_eq!(config, RepoSource::Base, folder.id, folder);
        assert_metadata_eq!(config, RepoSource::Base, document.id, document);
        assert_metadata_eq!(config, RepoSource::Base, document2.id, document2);
        assert_metadata_eq!(config, RepoSource::Base, document3.id, document3);
        assert_document_eq!(config, RepoSource::Base, document.id, b"document content 2");
        assert_document_eq!(
            config,
            RepoSource::Base,
            document2.id,
            b"document 2 content"
        );
        assert_document_eq!(
            config,
            RepoSource::Base,
            document3.id,
            b"document 3 content"
        );
        assert_metadata_count!(config, RepoSource::Base, 5);
        assert_metadata_count!(config, RepoSource::Local, 5);
        assert_document_count!(config, RepoSource::Base, 3);
        assert_document_count!(config, RepoSource::Local, 3);
    }

    #[test]
    fn prune_deleted() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);

        document.deleted = true;
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_nonexistent!(config, RepoSource::Base, document.id);
        assert_metadata_nonexistent!(config, RepoSource::Local, document.id);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn prune_deleted_document_edit() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        document.deleted = true;
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Local, &document, b"document content 2")
            .unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_nonexistent!(config, RepoSource::Base, document.id);
        assert_metadata_nonexistent!(config, RepoSource::Local, document.id);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn prune_deleted_document_in_deleted_folder() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let mut folder =
            file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let document =
            file_service::create(FileType::Document, folder.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        folder.deleted = true;
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_nonexistent!(config, RepoSource::Base, folder.id);
        assert_metadata_nonexistent!(config, RepoSource::Local, folder.id);
        assert_metadata_nonexistent!(config, RepoSource::Base, document.id);
        assert_metadata_nonexistent!(config, RepoSource::Local, document.id);
        assert_metadata_count!(config, RepoSource::Base, 1);
        assert_metadata_count!(config, RepoSource::Local, 1);
        assert_document_count!(config, RepoSource::Base, 0);
        assert_document_count!(config, RepoSource::Local, 0);
    }

    #[test]
    fn prune_deleted_document_moved_from_deleted_folder() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let mut folder =
            file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let mut document =
            file_service::create(FileType::Document, folder.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        folder.deleted = true;
        document.parent = root.id;
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document).unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_nonexistent!(config, RepoSource::Base, folder.id);
        assert_metadata_nonexistent!(config, RepoSource::Local, folder.id);
        assert_metadata_eq!(config, RepoSource::Base, document.id, document);
        assert_metadata_eq!(config, RepoSource::Local, document.id, document);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn prune_deleted_base_only() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let mut document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        let mut document_local = document.clone();
        document_local.decrypted_name = String::from("renamed document");
        file_repo::insert_metadata(config, RepoSource::Local, &document_local).unwrap();
        document.deleted = true;
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert_metadata_eq!(config, RepoSource::Base, document.id, document);
        assert_metadata_eq!(config, RepoSource::Local, document.id, document_local);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn prune_deleted_local_only() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let document =
            file_service::create(FileType::Document, root.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        let mut document_deleted = document.clone();
        document_deleted.deleted = true;
        file_repo::insert_metadata(config, RepoSource::Local, &document_deleted).unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert_metadata_eq!(config, RepoSource::Base, document.id, document);
        assert_metadata_eq!(config, RepoSource::Local, document.id, document_deleted);
        assert_metadata_count!(config, RepoSource::Base, 2);
        assert_metadata_count!(config, RepoSource::Local, 2);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }

    #[test]
    fn prune_deleted_document_moved_from_deleted_folder_local_only() {
        let config = &temp_config();
        let account = test_utils::generate_account();
        let root = file_service::create_root(&account.username);
        let folder = file_service::create(FileType::Folder, root.id, "folder", &account.username);
        let document =
            file_service::create(FileType::Document, folder.id, "document", &account.username);

        account_repo::insert(config, &account).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &root).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &folder).unwrap();
        file_repo::insert_metadata(config, RepoSource::Base, &document).unwrap();
        file_repo::insert_document(config, RepoSource::Base, &document, b"document content")
            .unwrap();

        assert_metadata_changes_count!(config, 0);
        assert_document_changes_count!(config, 0);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);

        let mut folder_deleted = folder.clone();
        folder_deleted.deleted = true;
        let mut document_moved = document.clone();
        document_moved.parent = root.id;
        file_repo::insert_metadata(config, RepoSource::Base, &folder_deleted).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &folder_deleted).unwrap();
        file_repo::insert_metadata(config, RepoSource::Local, &document_moved).unwrap();
        file_repo::prune_deleted(config).unwrap();

        assert_metadata_changes_count!(config, 1);
        assert_document_changes_count!(config, 0);
        assert_metadata_eq!(config, RepoSource::Base, document.id, document);
        assert_metadata_eq!(config, RepoSource::Local, document.id, document_moved);
        assert_metadata_count!(config, RepoSource::Base, 3);
        assert_metadata_count!(config, RepoSource::Local, 3);
        assert_document_count!(config, RepoSource::Base, 1);
        assert_document_count!(config, RepoSource::Local, 1);
    }
}
