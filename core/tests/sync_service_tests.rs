mod integration_test;

#[cfg(test)]
mod sync_tests {
    use crate::integration_test::{generate_account, test_db};
    use lockbook_core::model::work_unit::WorkUnit;
    use lockbook_core::repo::file_metadata_repo::FileMetadataRepo;
    use lockbook_core::service::account_service::AccountService;
    use lockbook_core::service::file_service::FileService;
    use lockbook_core::service::sync_service::SyncService;
    use lockbook_core::{
        DefaultAccountService, DefaultFileMetadataRepo, DefaultFileService, DefaultSyncService,
    };

    #[test]
    fn test_create_files_and_folders_sync() {
        let generated_account = generate_account();
        let db = test_db();
        let account = DefaultAccountService::create_account(
            &db,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        assert_eq!(
            DefaultSyncService::calculate_work(&db)
                .unwrap()
                .work_units
                .len(),
            0
        );

        DefaultFileService::create_at_path(
            &db,
            format!("{}/a/b/c/test", account.username).as_str(),
        )
        .unwrap();

        assert_eq!(
            DefaultSyncService::calculate_work(&db)
                .unwrap()
                .work_units
                .len(),
            4
        );

        assert!(DefaultSyncService::sync(&db).is_ok());

        let db2 = test_db();
        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db).unwrap(),
        )
        .unwrap();

        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            5
        );

        DefaultSyncService::sync(&db2).unwrap();
        assert_eq!(
            DefaultFileMetadataRepo::get_all(&db).unwrap(),
            DefaultFileMetadataRepo::get_all(&db2).unwrap()
        );

        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            0
        );
    }

    #[test]
    fn test_edit_document_sync() {
        let generated_account = generate_account();
        let db = test_db();
        let account = DefaultAccountService::create_account(
            &db,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        assert_eq!(
            DefaultSyncService::calculate_work(&db)
                .unwrap()
                .work_units
                .len(),
            0
        );
        println!("1st calculate work");

        let file = DefaultFileService::create_at_path(
            &db,
            format!("{}/a/b/c/test", account.username).as_str(),
        )
        .unwrap();

        assert!(DefaultSyncService::sync(&db).is_ok());
        println!("1st sync done");

        let db2 = test_db();
        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db).unwrap(),
        )
        .unwrap();

        DefaultSyncService::sync(&db2).unwrap();
        println!("2nd sync done, db2");

        DefaultFileService::write_document(&db, file.id, "meaningful messages".as_bytes()).unwrap();

        assert_eq!(
            DefaultSyncService::calculate_work(&db)
                .unwrap()
                .work_units
                .len(),
            1
        );
        println!("2nd calculate work, db1, 1 dirty file");

        match DefaultSyncService::calculate_work(&db)
            .unwrap()
            .work_units
            .get(0)
            .unwrap()
            .clone()
        {
            WorkUnit::LocalChange { metadata } => assert_eq!(metadata.name, file.name),
            WorkUnit::ServerChange { .. } => {
                panic!("This should have been a local change with no server changes!")
            }
        };
        println!("3rd calculate work, db1, 1 dirty file");

        DefaultSyncService::sync(&db).unwrap();
        println!("3rd sync done, db1, dirty file pushed");

        assert_eq!(
            DefaultSyncService::calculate_work(&db)
                .unwrap()
                .work_units
                .len(),
            0
        );
        println!("4th calculate work, db1, dirty file pushed");

        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            1
        );
        println!("5th calculate work, db2, dirty file needs to be pulled");

        let edited_file = DefaultFileMetadataRepo::get(&db, file.id).unwrap();

        match DefaultSyncService::calculate_work(&db2)
            .unwrap()
            .work_units
            .get(0)
            .unwrap()
            .clone()
        {
            WorkUnit::ServerChange { metadata } => assert_eq!(metadata, edited_file),
            WorkUnit::LocalChange { .. } => {
                panic!("This should have been a ServerChange with no LocalChange!")
            }
        };
        println!("6th calculate work, db2, dirty file needs to be pulled");

        DefaultSyncService::sync(&db2).unwrap();
        println!("4th sync done, db2, dirty file pulled");
        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            0
        );
        println!("7th calculate work ");

        assert_eq!(
            DefaultFileService::read_document(&db2, edited_file.id).unwrap(),
            "meaningful messages".as_bytes()
        );
        assert_eq!(&db.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_move_document_sync() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/folder1/test.txt", account.username),
        )
        .unwrap();

        DefaultFileService::write_document(&db1, file.id, "nice document".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();

        DefaultSyncService::sync(&db2).unwrap();

        assert_eq!(
            DefaultFileMetadataRepo::get_all(&db1).unwrap(),
            DefaultFileMetadataRepo::get_all(&db2).unwrap()
        );
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());

        let new_folder =
            DefaultFileService::create_at_path(&db1, &format!("{}/folder2/", account.username))
                .unwrap();

        DefaultFileService::move_file(&db1, file.id, new_folder.id).unwrap();
        assert_eq!(
            DefaultSyncService::calculate_work(&db1)
                .unwrap()
                .work_units
                .len(),
            2
        );
        assert_ne!(&db1.checksum().unwrap(), &db2.checksum().unwrap());

        DefaultSyncService::sync(&db1).unwrap();
        assert_eq!(
            DefaultSyncService::calculate_work(&db1)
                .unwrap()
                .work_units
                .len(),
            0
        );

        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            2
        );
        DefaultSyncService::sync(&db2).unwrap();
        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            0
        );
        assert_eq!(
            DefaultFileMetadataRepo::get_all(&db1).unwrap(),
            DefaultFileMetadataRepo::get_all(&db2).unwrap()
        );

        assert_eq!(
            DefaultFileService::read_document(&db2, file.id).unwrap(),
            "nice document".as_bytes()
        );

        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_move_reject() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/folder1/test.txt", account.username),
        )
        .unwrap();

        DefaultFileService::write_document(&db1, file.id, "Wow, what a doc".as_bytes()).unwrap();

        let new_folder1 =
            DefaultFileService::create_at_path(&db1, &format!("{}/folder2/", account.username))
                .unwrap();

        let new_folder2 =
            DefaultFileService::create_at_path(&db1, &format!("{}/folder3/", account.username))
                .unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();

        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::move_file(&db2, file.id, new_folder1.id).unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::move_file(&db1, file.id, new_folder2.id).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert_eq!(
            DefaultFileMetadataRepo::get_all(&db1).unwrap(),
            DefaultFileMetadataRepo::get_all(&db2).unwrap()
        );

        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());

        assert_eq!(
            DefaultFileMetadataRepo::get(&db1, file.id).unwrap().parent,
            new_folder1.id
        );
        assert_eq!(
            DefaultFileService::read_document(&db2, file.id).unwrap(),
            "Wow, what a doc".as_bytes()
        );
    }

    #[test]
    fn test_rename_sync() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/folder1/test.txt", account.username),
        )
        .unwrap();

        DefaultFileService::rename_file(&db1, file.parent, "folder1-new").unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        assert_eq!(
            DefaultFileMetadataRepo::get_by_path(
                &db2,
                &format!("{}/folder1-new", account.username)
            )
            .unwrap()
            .unwrap()
            .name,
            "folder1-new"
        );
        assert_eq!(
            DefaultFileMetadataRepo::get_by_path(
                &db2,
                &format!("{}/folder1-new/", account.username)
            )
            .unwrap()
            .unwrap()
            .name,
            "folder1-new"
        );
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_rename_reject_sync() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/folder1/test.txt", account.username),
        )
        .unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        DefaultFileService::rename_file(&db1, file.parent, "folder1-new").unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();
        DefaultFileService::rename_file(&db2, file.parent, "folder2-new").unwrap();
        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert_eq!(
            DefaultFileMetadataRepo::get_by_path(
                &db2,
                &format!("{}/folder2-new", account.username)
            )
            .unwrap()
            .unwrap()
            .name,
            "folder2-new"
        );
        assert_eq!(
            DefaultFileMetadataRepo::get_by_path(
                &db2,
                &format!("{}/folder2-new/", account.username)
            )
            .unwrap()
            .unwrap()
            .name,
            "folder2-new"
        );
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn move_then_edit() {
        let db1 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file =
            DefaultFileService::create_at_path(&db1, &format!("{}/test.txt", account.username))
                .unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultFileService::rename_file(&db1, file.id, "new_name.txt").unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultFileService::write_document(&db1, file.id, "noice".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();
    }

    #[test]
    fn sync_fs_invalid_state_via_rename() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file1 =
            DefaultFileService::create_at_path(&db1, &format!("{}/test.txt", account.username))
                .unwrap();
        let file2 =
            DefaultFileService::create_at_path(&db1, &format!("{}/test2.txt", account.username))
                .unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::rename_file(&db2, file1.id, "test3.txt").unwrap();

        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::rename_file(&db1, file2.id, "test3.txt").unwrap();
        // Just operate on the server work
        DefaultSyncService::calculate_work(&db1)
            .unwrap()
            .work_units
            .into_iter()
            .filter(|work| match work {
                WorkUnit::LocalChange { .. } => false,
                WorkUnit::ServerChange { .. } => true,
            })
            .for_each(|work| DefaultSyncService::execute_work(&db1, &account, work).unwrap());

        println!(
            "{:#?}",
            DefaultFileMetadataRepo::test_repo_integrity(&db1).unwrap()
        );

        assert!(DefaultFileMetadataRepo::test_repo_integrity(&db1)
            .unwrap()
            .is_empty());

        assert_eq!(
            DefaultSyncService::calculate_work(&db1)
                .unwrap()
                .work_units
                .len(),
            1
        );

        DefaultSyncService::sync(&db1).unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        assert_eq!(
            DefaultFileMetadataRepo::get_all(&db1).unwrap(),
            DefaultFileMetadataRepo::get_all(&db2).unwrap()
        );

        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn sync_fs_invalid_state_via_move() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file1 =
            DefaultFileService::create_at_path(&db1, &format!("{}/a/test.txt", account.username))
                .unwrap();
        let file2 =
            DefaultFileService::create_at_path(&db1, &format!("{}/b/test.txt", account.username))
                .unwrap();

        DefaultSyncService::sync(&db1).unwrap();
        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::move_file(
            &db1,
            file1.id,
            DefaultFileMetadataRepo::get_root(&db1).unwrap().unwrap().id,
        )
        .unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        DefaultFileService::move_file(
            &db2,
            file2.id,
            DefaultFileMetadataRepo::get_root(&db2).unwrap().unwrap().id,
        )
        .unwrap();

        println!("{:#?}", DefaultFileMetadataRepo::get_all(&db2).unwrap());

        DefaultSyncService::calculate_work(&db2)
            .unwrap()
            .work_units
            .into_iter()
            .filter(|work| match work {
                WorkUnit::LocalChange { .. } => false,
                WorkUnit::ServerChange { .. } => true,
            })
            .for_each(|work| DefaultSyncService::execute_work(&db2, &account, work).unwrap());

        println!("{:#?}", DefaultFileMetadataRepo::get_all(&db2).unwrap());

        assert!(DefaultFileMetadataRepo::test_repo_integrity(&db2)
            .unwrap()
            .is_empty());

        assert_eq!(
            DefaultSyncService::calculate_work(&db1)
                .unwrap()
                .work_units
                .len(),
            0
        );

        assert_eq!(
            DefaultSyncService::calculate_work(&db2)
                .unwrap()
                .work_units
                .len(),
            1
        );

        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert_eq!(
            DefaultFileMetadataRepo::get_all(&db1).unwrap(),
            DefaultFileMetadataRepo::get_all(&db2).unwrap()
        );

        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_content_conflict_unmergable() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file =
            DefaultFileService::create_at_path(&db1, &format!("{}/test.bin", account.username))
                .unwrap();

        DefaultFileService::write_document(&db1, file.id, "some good content".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::write_document(&db1, file.id, "some new content".as_bytes()).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        DefaultFileService::write_document(&db2, file.id, "some offline content".as_bytes())
            .unwrap();
        let works = DefaultSyncService::calculate_work(&db2).unwrap();

        assert_eq!(works.work_units.len(), 2);

        for work in works.clone().work_units {
            DefaultSyncService::execute_work(&db2, &account, work).unwrap();
        }

        let works = DefaultSyncService::calculate_work(&db2).unwrap();
        assert_eq!(works.work_units.len(), 1);

        match works.work_units.get(0).unwrap() {
            WorkUnit::LocalChange { metadata } => {
                assert!(metadata.name.contains("CONTENT-CONFLICT"))
            }
            WorkUnit::ServerChange { .. } => panic!("This should not be the work type"),
        }

        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_content_conflict_mergable() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/mergable_file.md", account.username),
        )
        .unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\n".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\nLine 2\n".as_bytes()).unwrap();
        DefaultSyncService::sync(&db1).unwrap();
        DefaultFileService::write_document(&db2, file.id, "Line 1\nOffline Line\n".as_bytes())
            .unwrap();

        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 1"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 2"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Offline Line"));
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_content_conflict_local_move_before_mergable() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/mergable_file.md", account.username),
        )
        .unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\n".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\nLine 2\n".as_bytes()).unwrap();
        DefaultSyncService::sync(&db1).unwrap();
        let folder =
            DefaultFileService::create_at_path(&db2, &format!("{}/folder1/", account.username))
                .unwrap();
        DefaultFileService::move_file(&db2, file.id, folder.id).unwrap();
        DefaultFileService::write_document(&db2, file.id, "Line 1\nOffline Line\n".as_bytes())
            .unwrap();

        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 1"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 2"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Offline Line"));
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_content_conflict_local_after_before_mergable() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/mergable_file.md", account.username),
        )
        .unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\n".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\nLine 2\n".as_bytes()).unwrap();
        DefaultSyncService::sync(&db1).unwrap();
        let folder =
            DefaultFileService::create_at_path(&db2, &format!("{}/folder1/", account.username))
                .unwrap();
        DefaultFileService::write_document(&db2, file.id, "Line 1\nOffline Line\n".as_bytes())
            .unwrap();
        DefaultFileService::move_file(&db2, file.id, folder.id).unwrap();

        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 1"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 2"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Offline Line"));
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_content_conflict_server_after_before_mergable() {
        let db1 = test_db();
        let db2 = test_db();

        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db1,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();
        let file = DefaultFileService::create_at_path(
            &db1,
            &format!("{}/mergable_file.md", account.username),
        )
        .unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\n".as_bytes()).unwrap();

        DefaultSyncService::sync(&db1).unwrap();

        DefaultAccountService::import_account(
            &db2,
            &DefaultAccountService::export_account(&db1).unwrap(),
        )
        .unwrap();
        DefaultSyncService::sync(&db2).unwrap();

        DefaultFileService::write_document(&db1, file.id, "Line 1\nLine 2\n".as_bytes()).unwrap();
        let folder =
            DefaultFileService::create_at_path(&db1, &format!("{}/folder1/", account.username))
                .unwrap();
        DefaultFileService::move_file(&db1, file.id, folder.id).unwrap();
        DefaultSyncService::sync(&db1).unwrap();
        DefaultFileService::write_document(&db2, file.id, "Line 1\nOffline Line\n".as_bytes())
            .unwrap();

        DefaultSyncService::sync(&db2).unwrap();
        DefaultSyncService::sync(&db1).unwrap();

        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 1"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Line 2"));
        assert!(String::from_utf8_lossy(
            &DefaultFileService::read_document(&db1, file.id).unwrap()
        )
        .contains("Offline Line"));
        assert_eq!(&db1.checksum().unwrap(), &db2.checksum().unwrap());
    }

    #[test]
    fn test_not_really_editing_should_not_cause_work() {
        let db = test_db();
        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file =
            DefaultFileService::create_at_path(&db, &format!("{}/file.md", account.username))
                .unwrap();

        DefaultFileService::write_document(&db, file.id, "original".as_bytes()).unwrap();

        DefaultSyncService::sync(&db).unwrap();

        assert!(DefaultSyncService::calculate_work(&db)
            .unwrap()
            .work_units
            .is_empty());

        DefaultFileService::write_document(&db, file.id, "original".as_bytes()).unwrap();

        assert_eq!(
            DefaultSyncService::calculate_work(&db)
                .unwrap()
                .work_units
                .len(),
            0
        );
    }

    #[test]
    fn test_not_really_renaming_should_not_cause_work() {
        let db = test_db();
        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file =
            DefaultFileService::create_at_path(&db, &format!("{}/file.md", account.username))
                .unwrap();

        DefaultSyncService::sync(&db).unwrap();

        assert!(DefaultSyncService::calculate_work(&db)
            .unwrap()
            .work_units
            .is_empty());

        assert!(DefaultFileService::rename_file(&db, file.id, "file.md").is_err())
    }

    #[test]
    fn test_not_really_moving_should_not_cause_work() {
        let db = test_db();
        let generated_account = generate_account();
        let account = DefaultAccountService::create_account(
            &db,
            &generated_account.username,
            &generated_account.api_url,
        )
        .unwrap();

        let file =
            DefaultFileService::create_at_path(&db, &format!("{}/file.md", account.username))
                .unwrap();

        DefaultSyncService::sync(&db).unwrap();

        assert!(DefaultSyncService::calculate_work(&db)
            .unwrap()
            .work_units
            .is_empty());

        assert!(DefaultFileService::move_file(&db, file.id, file.parent).is_err())
    }
}
