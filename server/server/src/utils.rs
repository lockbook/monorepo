use crate::ServerError;
use lockbook_models::api::{GetBuildInfoError, GetBuildInfoResponse};

use crate::config::Config;
use google_androidpublisher3::{hyper, hyper_rustls};
use shadow_rs::shadow;

shadow!(build_info);

pub fn username_is_valid(username: &str) -> bool {
    !username.is_empty()
        && username
            .to_lowercase()
            .chars()
            .all(|c| ('a'..='z').contains(&c) || ('0'..='9').contains(&c))
}

pub fn get_build_info() -> Result<GetBuildInfoResponse, ServerError<GetBuildInfoError>> {
    Ok(GetBuildInfoResponse {
        build_version: env!("CARGO_PKG_VERSION"),
        git_commit_hash: build_info::COMMIT_HASH,
    })
}

pub async fn get_android_client(config: &Config) -> google_androidpublisher3::AndroidPublisher {
    let auth = match &config.google.service_account_cred_path {
        None => google_androidpublisher3::oauth2::InstalledFlowAuthenticator::builder(
            google_androidpublisher3::oauth2::ApplicationSecret::default(),
            google_androidpublisher3::oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await
        .unwrap(),
        Some(cred_path) => {
            let service_account_key: google_androidpublisher3::oauth2::ServiceAccountKey =
                google_androidpublisher3::oauth2::read_service_account_key(cred_path)
                    .await
                    .unwrap();

            google_androidpublisher3::oauth2::ServiceAccountAuthenticator::builder(
                service_account_key,
            )
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
