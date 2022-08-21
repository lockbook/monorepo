use lockbook_core::Core;
use lockbook_shared::file::ShareMode;
use test_utils::*;

/// Tests that setup two synced devices, operate on both devices, then sync both twice (work
/// should be none, devices dbs should be equal, deleted files should be pruned).

fn sync_and_assert(c1: &Core, c2: &Core) {
    c1.sync(None).unwrap();
    c2.sync(None).unwrap();
    c1.sync(None).unwrap();
    c2.sync(None).unwrap();

    c1.validate().unwrap();
    assert::cores_equal(c1, c2);
    assert::local_work_paths(c1, &[]);
    assert::server_work_paths(c1, &[]);
    assert::deleted_files_pruned(c1);
}

#[test]
fn identical_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/document", "/parent/").unwrap();
    move_by_path(&c2, "/document", "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document"]);
    assert::all_document_contents(&c2, &[("/parent/document", b"")]);
}

#[test]
fn different_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/parent2/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/document", "/parent/").unwrap();
    move_by_path(&c2, "/document", "/parent2/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent2/", "/parent/document"]);
    assert::all_document_contents(&c2, &[("/parent/document", b"")]);
}

#[test]
fn identical_rename() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/document", "document2").unwrap();
    rename_path(&c2, "/document", "document2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document2"]);
    assert::all_document_contents(&c2, &[("/document2", b"")]);
}

#[test]
fn different_rename() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/document", "document2").unwrap();
    rename_path(&c2, "/document", "document3").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document2"]);
    assert::all_document_contents(&c2, &[("/document2", b"")]);
}

#[test]
fn move_then_rename() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/document", "/parent/").unwrap();
    rename_path(&c2, "/document", "document2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document2"]);
    assert::all_document_contents(&c2, &[("/parent/document2", b"")]);
}

#[test]
fn rename_then_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/document", "document2").unwrap();
    move_by_path(&c2, "/document", "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document2"]);
    assert::all_document_contents(&c2, &[("/parent/document2", b"")]);
}

#[test]
fn identical_delete() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/document").unwrap();
    delete_path(&c2, "/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn identical_delete_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/").unwrap();
    delete_path(&c2, "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_parent_then_direct() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/").unwrap();
    delete_path(&c2, "/parent/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_direct_then_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/document").unwrap();
    delete_path(&c2, "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn identical_delete_grandparent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/").unwrap();
    delete_path(&c2, "/grandparent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_grandparent_then_direct() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/").unwrap();
    delete_path(&c2, "/grandparent/parent/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_direct_then_grandparent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/parent/document").unwrap();
    delete_path(&c2, "/grandparent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_grandparent_then_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/").unwrap();
    delete_path(&c2, "/grandparent/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_parent_then_grandparent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/parent/").unwrap();
    delete_path(&c2, "/grandparent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn move_then_delete() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/document", "/parent/").unwrap();
    delete_path(&c2, "/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_then_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/document").unwrap();
    move_by_path(&c2, "/document", "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn move_then_delete_new_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/document", "/parent/").unwrap();
    delete_path(&c2, "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_new_parent_then_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/").unwrap();
    move_by_path(&c2, "/document", "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn move_then_delete_old_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/parent/document", "").unwrap();
    delete_path(&c2, "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document"]);
    assert::all_document_contents(&c2, &[("/document", b"")]);
}

#[test]
fn delete_old_parent_then_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/").unwrap();
    move_by_path(&c2, "/parent/document", "").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn rename_then_delete() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/document", "document2").unwrap();
    delete_path(&c2, "/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_then_rename() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/document").unwrap();
    rename_path(&c2, "/document", "document2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn create_then_move_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/parent2/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    c1.create_at_path("/parent/document").unwrap();
    move_by_path(&c2, "/parent/", "/parent2/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent2/", "/parent2/parent/", "/parent2/parent/document"]);
    assert::all_document_contents(&c2, &[("/parent2/parent/document", b"")]);
}

#[test]
fn move_parent_then_create() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/parent2/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/parent/", "/parent2/").unwrap();
    c2.create_at_path("/parent/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent2/", "/parent2/parent/", "/parent2/parent/document"]);
    assert::all_document_contents(&c2, &[("/parent2/parent/document", b"")]);
}

#[test]
fn create_then_rename_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    c1.create_at_path("/parent/document").unwrap();
    rename_path(&c2, "/parent/", "parent2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent2/", "/parent2/document"]);
    assert::all_document_contents(&c2, &[("/parent2/document", b"")]);
}

#[test]
fn rename_parent_then_create() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/parent/", "parent2").unwrap();
    c2.create_at_path("/parent/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent2/", "/parent2/document"]);
    assert::all_document_contents(&c2, &[("/parent2/document", b"")]);
}

#[test]
fn create_then_delete_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    c1.create_at_path("/parent/document").unwrap();
    delete_path(&c2, "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_parent_then_create() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/").unwrap();
    c2.create_at_path("/parent/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn create_then_delete_grandparent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    c1.create_at_path("/grandparent/parent/document").unwrap();
    delete_path(&c2, "/grandparent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_grandparent_then_create() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/").unwrap();
    c2.create_at_path("/grandparent/parent/document").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn identical_content_edit_not_mergable() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.draw").unwrap();
    write_path(&c1, "/document.draw", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.draw", b"document content 2").unwrap();
    write_path(&c2, "/document.draw", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document.draw", "/document-1.draw"]);
    assert::all_document_contents(
        &c2,
        &[("/document.draw", b"document content 2"), ("/document-1.draw", b"document content 2")],
    );
}

#[test]
fn identical_content_edit_mergable() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document content 2").unwrap();
    write_path(&c2, "/document.md", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document.md"]);
    assert::all_document_contents(&c2, &[("/document.md", b"document content 2")]);
}

#[test]
fn different_content_edit_not_mergable() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.draw").unwrap();
    write_path(&c1, "/document.draw", b"document\n\ncontent\n").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.draw", b"document 2\n\ncontent\n").unwrap();
    write_path(&c2, "/document.draw", b"document\n\ncontent 2\n").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document.draw", "/document-1.draw"]);
    assert::all_document_contents(
        &c2,
        &[
            ("/document.draw", b"document 2\n\ncontent\n"),
            ("/document-1.draw", b"document\n\ncontent 2\n"),
        ],
    );
}

#[test]
fn different_content_edit_mergable() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document\n\ncontent\n").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document 2\n\ncontent\n").unwrap();
    write_path(&c2, "/document.md", b"document\n\ncontent 2\n").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document.md"]);
    assert::all_document_contents(&c2, &[("/document.md", b"document 2\n\ncontent 2\n")]);
}

#[test]
fn different_content_edit_mergable_with_move_in_first_sync() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document\n\ncontent\n").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document 2\n\ncontent\n").unwrap();
    move_by_path(&c1, "/document.md", "/parent/").unwrap();
    write_path(&c2, "/document.md", b"document\n\ncontent 2\n").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document.md"]);
    assert::all_document_contents(&c2, &[("/parent/document.md", b"document 2\n\ncontent 2\n")]);
}

#[test]
fn different_content_edit_mergable_with_move_in_second_sync() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document\n\ncontent\n").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document 2\n\ncontent\n").unwrap();
    write_path(&c2, "/document.md", b"document\n\ncontent 2\n").unwrap();
    move_by_path(&c2, "/document.md", "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document.md"]);
    assert::all_document_contents(&c2, &[("/parent/document.md", b"document 2\n\ncontent 2\n")]);
}

#[test]
fn move_then_edit_content() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/document.md", "/parent/").unwrap();
    write_path(&c2, "/document.md", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document.md"]);
    assert::all_document_contents(&c2, &[("/parent/document.md", b"document content 2")]);
}

#[test]
fn edit_content_then_move() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/").unwrap();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document content 2").unwrap();
    move_by_path(&c2, "/document.md", "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/parent/", "/parent/document.md"]);
    assert::all_document_contents(&c2, &[("/parent/document.md", b"document content 2")]);
}

#[test]
fn rename_then_edit_content() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/document.md", "document2.md").unwrap();
    write_path(&c2, "/document.md", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document2.md"]);
    assert::all_document_contents(&c2, &[("/document2.md", b"document content 2")]);
}

#[test]
fn edit_content_then_rename() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document content 2").unwrap();
    rename_path(&c2, "/document.md", "document2.md").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/", "/document2.md"]);
    assert::all_document_contents(&c2, &[("/document2.md", b"document content 2")]);
}

#[test]
fn delete_then_edit_content() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/document.md").unwrap();
    write_path(&c2, "/document.md", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn edit_content_then_delete() {
    let c1 = test_core_with_account();
    c1.create_at_path("/document.md").unwrap();
    write_path(&c1, "/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/document.md", b"document content 2").unwrap();
    delete_path(&c2, "/document.md").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_parent_then_edit_content() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document.md").unwrap();
    write_path(&c1, "/parent/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/parent/").unwrap();
    write_path(&c2, "/parent/document.md", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn edit_content_then_delete_parent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/parent/document.md").unwrap();
    write_path(&c1, "/parent/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/parent/document.md", b"document content 2").unwrap();
    delete_path(&c2, "/parent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn delete_grandparent_then_edit_content() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document.md")
        .unwrap();
    write_path(&c1, "/grandparent/parent/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    delete_path(&c1, "/grandparent/").unwrap();
    write_path(&c2, "/grandparent/parent/document.md", b"document content 2").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn edit_content_then_delete_grandparent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/grandparent/parent/document.md")
        .unwrap();
    write_path(&c1, "/grandparent/parent/document.md", b"document content").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    write_path(&c1, "/grandparent/parent/document.md", b"document content 2").unwrap();
    delete_path(&c2, "/grandparent/").unwrap();

    sync_and_assert(&c1, &c2);
    assert::all_paths(&c2, &["/"]);
    assert::all_document_contents(&c2, &[]);
}

#[test]
fn create_two_links() {
    let mut cores = vec![vec![test_core_with_account()], vec![test_core_with_account()]];
    let new_client = another_client(&cores[1][0]);
    cores[1].push(new_client);
    let accounts = cores
        .iter()
        .map(|cores| cores[0].get_account().unwrap())
        .collect::<Vec<_>>();

    let document = cores[0][0].create_at_path("/document").unwrap();
    cores[0][0]
        .share_file(document.id, &accounts[1].username, ShareMode::Read)
        .unwrap();
    cores[0][0].sync(None).unwrap();

    cores[1][0].sync(None).unwrap();
    cores[1][0]
        .create_link_at_path("/link1", document.id)
        .unwrap();

    cores[1][1].sync(None).unwrap();
    cores[1][1]
        .create_link_at_path("/link2", document.id)
        .unwrap();

    sync_and_assert(&cores[1][0], &cores[1][1]);
    assert::all_paths(&cores[1][0], &["/", "/link1"]);
}

#[test]
fn share_then_create_link_in_folder() {
    let mut cores = vec![vec![test_core_with_account()], vec![test_core_with_account()]];
    let new_client = another_client(&cores[1][0]);
    cores[1].push(new_client);
    let accounts = cores
        .iter()
        .map(|cores| cores[0].get_account().unwrap())
        .collect::<Vec<_>>();

    let document = cores[0][0].create_at_path("/document").unwrap();
    cores[0][0]
        .share_file(document.id, &accounts[1].username, ShareMode::Read)
        .unwrap();
    cores[0][0].sync(None).unwrap();

    let folder = cores[1][0].create_at_path("/folder/").unwrap();
    cores[1][0].sync(None).unwrap();
    cores[1][0]
        .share_file(folder.id, &accounts[0].username, ShareMode::Read)
        .unwrap();

    cores[1][1].sync(None).unwrap();
    cores[1][1]
        .create_link_at_path("/folder/link", document.id)
        .unwrap();

    sync_and_assert(&cores[1][0], &cores[1][1]);
    assert::all_paths(&cores[1][0], &["/", "/folder/"]);
}

#[test]
fn create_link_in_folder_then_share() {
    let mut cores = vec![vec![test_core_with_account()], vec![test_core_with_account()]];
    let new_client = another_client(&cores[1][0]);
    cores[1].push(new_client);
    let accounts = cores
        .iter()
        .map(|cores| cores[0].get_account().unwrap())
        .collect::<Vec<_>>();

    let document = cores[0][0].create_at_path("/document").unwrap();
    cores[0][0]
        .share_file(document.id, &accounts[1].username, ShareMode::Read)
        .unwrap();
    cores[0][0].sync(None).unwrap();

    let folder = cores[1][0].create_at_path("/folder/").unwrap();
    cores[1][0].sync(None).unwrap();
    cores[1][0]
        .share_file(folder.id, &accounts[0].username, ShareMode::Read)
        .unwrap();

    cores[1][1].sync(None).unwrap();
    cores[1][1]
        .create_link_at_path("/folder/link", document.id)
        .unwrap();

    sync_and_assert(&cores[1][1], &cores[1][0]); // note: reverse order from above test
    assert::all_paths(&cores[1][0], &["/", "/folder/", "/folder/link"]);
}

#[test]
fn create_link_then_move_to_owned_folder() {}

#[test]
fn create_link_then_delete() {}
