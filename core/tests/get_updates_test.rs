mod integration_test;

#[cfg(test)]
mod get_updates_test {
    use crate::integration_test::{generate_account, generate_root_metadata};
    use lockbook_core::client::Client;
    use lockbook_core::model::api::{GetUpdatesRequest, NewAccountRequest};
    use lockbook_core::DefaultClient;

    #[test]
    fn get_updates() {
        // new account
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);
        DefaultClient::request(
            &account.api_url,
            &account.private_key,
            NewAccountRequest::new(&account, &root),
        )
        .unwrap();

        // get updates at version 0
        let result = DefaultClient::request(
            &account.api_url,
            &account.private_key,
            GetUpdatesRequest {
                since_metadata_version: 0,
            },
        )
        .unwrap()
        .file_metadata
        .len();
        assert_eq!(result, 1);

        // get updates at version of root folder
        let result = DefaultClient::request(
            &account.api_url,
            &account.private_key,
            GetUpdatesRequest {
                since_metadata_version: root.metadata_version,
            },
        )
        .unwrap()
        .file_metadata
        .len();
        assert_eq!(result, 0);
    }
}
