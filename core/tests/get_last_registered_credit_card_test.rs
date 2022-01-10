mod integration_test;

#[cfg(test)]
mod get_last_registered_credit_card_test {
    use lockbook_core::assert_matches;
    use lockbook_core::service::api_service;
    use lockbook_core::service::api_service::ApiError;
    use lockbook_core::service::test_utils::{
        generate_account, generate_monthly_account_tier, generate_root_metadata, test_credit_cards,
    };
    use lockbook_models::api::*;

    #[test]
    fn get_last_registered_credit_card() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);

        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        api_service::request(
            &account,
            SwitchAccountTierRequest {
                account_tier: generate_monthly_account_tier(
                    test_credit_cards::NO_AUTHENTICATION,
                    None,
                    None,
                    None,
                ),
            },
        )
        .unwrap();

        let result = api_service::request(&account, GetLastRegisteredCreditCardRequest {})
            .unwrap()
            .credit_card_last_4_digits;

        assert_matches!(result.as_str(), test_credit_cards::NO_AUTHENTICATION_LAST_4);
    }

    #[test]
    fn get_last_registered_credit_card_does_not_exist() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);

        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        let result = api_service::request(&account, GetLastRegisteredCreditCardRequest {});

        assert_matches!(
            result,
            Err(ApiError::<GetLastRegisteredCreditCardError>::Endpoint(
                GetLastRegisteredCreditCardError::OldCardDoesNotExist
            ))
        );
    }
}
