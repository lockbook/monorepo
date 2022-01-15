#[cfg(test)]
mod get_public_key_tests {
    use lockbook_core::assert_matches;
    use lockbook_core::service::api_service;
    use lockbook_core::service::api_service::ApiError;
    use lockbook_core::service::test_utils::{generate_account, generate_root_metadata};
    use lockbook_models::api::*;

    #[test]
    fn get_public_key() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);
        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        let result = api_service::request(
            &account,
            GetPublicKeyRequest {
                username: account.username.clone(),
            },
        )
        .unwrap()
        .key;
        assert_eq!(result, account.public_key());
    }

    #[test]
    fn get_public_key_not_found() {
        let account = generate_account();

        let result = api_service::request(
            &account,
            GetPublicKeyRequest {
                username: account.username.clone(),
            },
        );
        assert_matches!(
            result,
            Err(ApiError::<GetPublicKeyError>::Endpoint(
                GetPublicKeyError::UserNotFound
            ))
        );
    }
}
