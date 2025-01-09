use std::{
    collections::HashMap,
    future::{Future, IntoFuture},
    path::PathBuf,
    sync::Arc,
};

use futures::executor::block_on;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::{
    logic::{crypto::DecryptedDocument, path_ops::Filter},
    model::{
        account::{Account, Username},
        api::{
            AccountFilter, AccountIdentifier, AccountInfo, AdminFileInfoResponse,
            AdminSetUserTierInfo, AdminValidateAccount, AdminValidateServer, ServerIndex,
            StripeAccountTier, SubscriptionInfo,
        },
        core_config::Config,
        errors::{LbResult, TestRepoError, Warning},
        file::{File, ShareMode},
        file_metadata::{DocumentHmac, FileType},
    },
    service::{
        activity::RankingWeights,
        import_export::{ExportFileInfo, ImportStatus},
        search::{SearchConfig, SearchResult},
        sync::{SyncProgress, SyncStatus},
        usage::{UsageItemMetric, UsageMetrics},
    },
};

#[derive(Clone)]
pub struct Lb {
    lb: crate::Lb,
    #[cfg(not(target_family = "wasm"))]
    rt: Arc<Runtime>,
}

impl Lb {
    #[cfg(target_family = "wasm")]
    pub fn init(config: Config) -> LbResult<Self> {
        let lb = block_on(crate::Lb::init(config))?;
        Ok(Self { lb })
    }

    #[cfg(target_family = "wasm")]
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        block_on(future)
    }

    #[cfg(not(target_family = "wasm"))]
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        self.rt.block_on(future)
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn init(config: Config) -> LbResult<Self> {
        let rt = Arc::new(Runtime::new().unwrap());
        let lb = rt.block_on(crate::Lb::init(config))?;
        Ok(Self { rt, lb })
    }

    pub fn create_account(
        &self, username: &str, api_url: &str, welcome_doc: bool,
    ) -> LbResult<Account> {
        self.block_on(self.lb.create_account(username, api_url, welcome_doc))
    }

    pub fn import_account(&self, key: &str, api_url: Option<&str>) -> LbResult<Account> {
        block_on(self.lb.import_account(key, api_url))
    }

    pub fn export_account_private_key(&self) -> LbResult<String> {
        self.lb.export_account_private_key_v1()
    }

    pub fn export_account_phrase(&self) -> LbResult<String> {
        self.lb.export_account_phrase()
    }

    pub fn export_account_qr(&self) -> LbResult<Vec<u8>> {
        self.lb.export_account_qr()
    }

    pub fn get_account(&self) -> LbResult<&Account> {
        self.lb.get_account()
    }

    pub fn get_config(&self) -> Config {
        self.lb.config.clone()
    }

    pub fn create_file(&self, name: &str, parent: &Uuid, file_type: FileType) -> LbResult<File> {
        self.block_on(self.lb.create_file(name, parent, file_type))
    }

    pub fn safe_write(
        &self, id: Uuid, old_hmac: Option<DocumentHmac>, content: Vec<u8>,
    ) -> LbResult<DocumentHmac> {
        block_on(self.lb.safe_write(id, old_hmac, content))
    }

    pub fn write_document(&self, id: Uuid, content: &[u8]) -> LbResult<()> {
        block_on(self.lb.write_document(id, content))
    }

    pub fn get_root(&self) -> LbResult<File> {
        block_on(self.lb.root())
    }

    pub fn get_children(&self, id: &Uuid) -> LbResult<Vec<File>> {
        block_on(self.lb.get_children(id))
    }

    pub fn get_and_get_children_recursively(&self, id: &Uuid) -> LbResult<Vec<File>> {
        self.block_on(self.lb.get_and_get_children_recursively(id))
    }

    pub fn get_file_by_id(&self, id: Uuid) -> LbResult<File> {
        block_on(self.lb.get_file_by_id(id))
    }

    pub fn delete_file(&self, id: &Uuid) -> LbResult<()> {
        block_on(self.lb.delete(id))
    }

    pub fn read_document(&self, id: Uuid) -> LbResult<DecryptedDocument> {
        block_on(self.lb.read_document(id))
    }

    pub fn read_document_with_hmac(
        &self, id: Uuid,
    ) -> LbResult<(Option<DocumentHmac>, DecryptedDocument)> {
        block_on(self.lb.read_document_with_hmac(id))
    }

    pub fn list_metadatas(&self) -> LbResult<Vec<File>> {
        block_on(self.lb.list_metadatas())
    }

    pub fn rename_file(&self, id: &Uuid, new_name: &str) -> LbResult<()> {
        block_on(self.lb.rename_file(id, new_name))
    }

    pub fn move_file(&self, id: &Uuid, new_parent: &Uuid) -> LbResult<()> {
        block_on(self.lb.move_file(id, new_parent))
    }

    pub fn share_file(&self, id: Uuid, username: &str, mode: ShareMode) -> LbResult<()> {
        block_on(self.lb.share_file(id, username, mode))
    }

    pub fn get_pending_shares(&self) -> LbResult<Vec<File>> {
        block_on(self.lb.get_pending_shares())
    }

    pub fn delete_pending_share(&self, id: &Uuid) -> LbResult<()> {
        block_on(async { self.lb.reject_share(id).await })
    }

    pub fn create_link_at_path(&self, path_and_name: &str, target_id: Uuid) -> LbResult<File> {
        self.block_on(self.lb.create_link_at_path(path_and_name, target_id))
    }

    pub fn create_at_path(&self, path_and_name: &str) -> LbResult<File> {
        block_on(self.lb.create_at_path(path_and_name))
    }

    pub fn get_by_path(&self, path: &str) -> LbResult<File> {
        block_on(self.lb.get_by_path(path))
    }

    pub fn get_path_by_id(&self, id: Uuid) -> LbResult<String> {
        block_on(self.lb.get_path_by_id(id))
    }

    pub fn list_paths(&self, filter: Option<Filter>) -> LbResult<Vec<String>> {
        block_on(self.lb.list_paths(filter))
    }

    pub fn get_local_changes(&self) -> LbResult<Vec<Uuid>> {
        block_on(async {
            let tx = self.lb.ro_tx().await;
            let db = tx.db();
            Ok(db.local_metadata.get().keys().copied().collect())
        })
    }

    pub fn calculate_work(&self) -> LbResult<SyncStatus> {
        block_on(self.lb.calculate_work())
    }

    pub fn sync(&self, f: Option<Box<dyn Fn(SyncProgress) + Send>>) -> LbResult<SyncStatus> {
        block_on(self.lb.sync(f))
    }

    pub fn get_last_synced(&self) -> LbResult<i64> {
        block_on(async {
            let tx = self.lb.ro_tx().await;
            let db = tx.db();
            Ok(db.last_synced.get().copied().unwrap_or(0))
        })
    }

    pub fn get_last_synced_human_string(&self) -> LbResult<String> {
        block_on(self.lb.get_last_synced_human())
    }

    pub fn get_timestamp_human_string(&self, timestamp: i64) -> String {
        self.lb.get_timestamp_human_string(timestamp)
    }

    pub fn suggested_docs(&self, settings: RankingWeights) -> LbResult<Vec<Uuid>> {
        block_on(self.lb.suggested_docs(settings))
    }

    // TODO: examine why the old get_usage does a bunch of things
    pub fn get_usage(&self) -> LbResult<UsageMetrics> {
        block_on(self.lb.get_usage())
    }

    pub fn get_uncompressed_usage_breakdown(&self) -> LbResult<HashMap<Uuid, usize>> {
        block_on(self.lb.get_uncompressed_usage_breakdown())
    }

    pub fn get_uncompressed_usage(&self) -> LbResult<UsageItemMetric> {
        block_on(self.lb.get_uncompressed_usage())
    }

    pub fn import_files<F: Fn(ImportStatus)>(
        &self, sources: &[PathBuf], dest: Uuid, update_status: &F,
    ) -> LbResult<()> {
        self.block_on(self.lb.import_files(sources, dest, update_status))
    }

    pub fn export_files(
        &self, id: Uuid, dest: PathBuf, edit: bool,
        export_progress: &Option<Box<dyn Fn(ExportFileInfo)>>,
    ) -> LbResult<()> {
        self.block_on(self.lb.export_file(id, dest, edit, export_progress))
    }

    pub fn search_file_paths(&self, input: &str) -> LbResult<Vec<SearchResult>> {
        self.block_on(async { self.lb.search(input, SearchConfig::Paths).await })
    }

    pub fn search(&self, input: &str, cfg: SearchConfig) -> LbResult<Vec<SearchResult>> {
        block_on(self.lb.search(input, cfg))
    }

    pub fn validate(&self) -> Result<Vec<Warning>, TestRepoError> {
        block_on(self.lb.test_repo_integrity())
    }

    pub fn upgrade_account_stripe(&self, account_tier: StripeAccountTier) -> LbResult<()> {
        self.block_on(self.lb.upgrade_account_stripe(account_tier))
    }

    pub fn upgrade_account_google_play(
        &self, purchase_token: &str, account_id: &str,
    ) -> LbResult<()> {
        block_on(
            self.lb
                .upgrade_account_google_play(purchase_token, account_id),
        )
    }

    pub fn upgrade_account_app_store(
        &self, original_transaction_id: String, app_account_token: String,
    ) -> LbResult<()> {
        block_on(
            self.lb
                .upgrade_account_app_store(original_transaction_id, app_account_token),
        )
    }

    pub fn cancel_subscription(&self) -> LbResult<()> {
        block_on(self.lb.cancel_subscription())
    }

    pub fn get_subscription_info(&self) -> LbResult<Option<SubscriptionInfo>> {
        block_on(self.lb.get_subscription_info())
    }

    pub fn delete_account(&self) -> LbResult<()> {
        block_on(self.lb.delete_account())
    }

    pub fn admin_disappear_account(&self, username: &str) -> LbResult<()> {
        block_on(self.lb.disappear_account(username))
    }

    pub fn admin_disappear_file(&self, id: Uuid) -> LbResult<()> {
        block_on(self.lb.disappear_file(id))
    }

    pub fn admin_list_users(&self, filter: Option<AccountFilter>) -> LbResult<Vec<Username>> {
        block_on(self.lb.list_users(filter))
    }

    pub fn admin_get_account_info(&self, identifier: AccountIdentifier) -> LbResult<AccountInfo> {
        block_on(self.lb.get_account_info(identifier))
    }

    pub fn admin_validate_account(&self, username: &str) -> LbResult<AdminValidateAccount> {
        block_on(self.lb.validate_account(username))
    }

    pub fn admin_validate_server(&self) -> LbResult<AdminValidateServer> {
        block_on(self.lb.validate_server())
    }

    pub fn admin_file_info(&self, id: Uuid) -> LbResult<AdminFileInfoResponse> {
        block_on(self.lb.file_info(id))
    }

    pub fn admin_rebuild_index(&self, index: ServerIndex) -> LbResult<()> {
        block_on(self.lb.rebuild_index(index))
    }

    pub fn admin_set_user_tier(&self, username: &str, info: AdminSetUserTierInfo) -> LbResult<()> {
        block_on(self.lb.set_user_tier(username, info))
    }

    pub fn debug_info(&self, os_info: String) -> String {
        self.block_on(self.lb.debug_info(os_info))
            .unwrap_or_else(|e| format!("failed to produce debug info: {:?}", e.to_string()))
    }
}
