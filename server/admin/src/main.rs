mod delete_account;
mod feature_flags;

use crate::delete_account::delete_account;
use crate::Subcommands::{DeleteAccount, Features};
use deadpool_redis::Pool;

use deadpool_redis::Runtime;
use lockbook_server_lib::config::Config;
use lockbook_server_lib::ServerState;

use crate::feature_flags::handle_feature_flag;
use google_androidpublisher3::{hyper, hyper_rustls};
use lockbook_server_lib::content::file_content_client;
use s3::bucket::Bucket;
use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "A utility for a lockbook server administrator.")]
enum Subcommands {
    /// Purge a user, and all their files from postgres & s3
    ///  *Note: This is intentionally left unexposed to give the user experience of deleting a user more
    /// thought. This includes thinking about being able to mark themselves as compromised and indicate to
    /// collaborators that certain files are potentially compromised. This could also involve us reaching out
    /// to services like Stripe / Apple / Google and terminating open subscriptions.
    /// Additionally deleted usernames should not be "freed". Usernames are a form of identity that's
    /// immutable, if a username is compromised or deleted, it is consumed forever, someone else cannot
    /// assume that identity.
    DeleteAccount { username: String },

    /// Features for lockbook server.
    Features {
        #[structopt(subcommand)]
        feature_flag: Option<FeatureFlag>,
    },
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "Toggleable features for lockbook server.")]
pub enum FeatureFlag {
    NewAccount {
        /// Enable or disable new accounts from being created.
        #[structopt(parse(try_from_str))]
        enable: bool,
    },
}

#[tokio::main]
async fn main() {
    let config = Config::from_env_vars();
    let (index_db_pool, files_db_client) = connect_to_state(&config).await;
    let stripe_client = stripe::Client::new(&config.stripe.stripe_secret);

    let android_publisher = get_android_client(&config).await;

    let server_state = ServerState {
        config,
        index_db_pool,
        stripe_client,
        files_db_client,
        android_publisher,
    };

    let ok = match Subcommands::from_args() {
        DeleteAccount { username: user } => delete_account(server_state, &user).await,
        Features { feature_flag } => handle_feature_flag(server_state, feature_flag).await,
    };

    if ok {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

async fn connect_to_state(config: &Config) -> (Pool, Bucket) {
    let index_db_pool = deadpool_redis::Config::from_url(&config.index_db.redis_url)
        .create_pool(Some(Runtime::Tokio1));
    let files_db = file_content_client::create_client(&config.files_db);
    (index_db_pool.unwrap(), files_db.unwrap())
}

async fn get_android_client(config: &Config) -> google_androidpublisher3::AndroidPublisher {
    let auth = match &config.google.service_account_cred_path {
        None => {
            google_androidpublisher3::oauth2::InstalledFlowAuthenticator::builder(
                google_androidpublisher3::oauth2::ApplicationSecret::default(),
                google_androidpublisher3::oauth2::InstalledFlowReturnMethod::HTTPRedirect,
            ).build().await.unwrap()
        },
        Some(cred_path) => {
            let service_account_key: google_androidpublisher3::oauth2::ServiceAccountKey =
                google_androidpublisher3::oauth2::read_service_account_key(
                    cred_path,
                )
                    .await
                    .unwrap();

            google_androidpublisher3::oauth2::ServiceAccountAuthenticator::builder(service_account_key)
                .build()
                .await
                .unwrap()
        }
    };

    let client = hyper::Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::with_native_roots(Default::default())
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build(),
    );

    google_androidpublisher3::AndroidPublisher::new(client, auth)
}