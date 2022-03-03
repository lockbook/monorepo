#[cfg(test)]
mod switch_account_tier_test {
    use lockbook_core::assert_matches;
    use lockbook_core::service::api_service::ApiError;
    use lockbook_core::service::test_utils::{
        generate_account, generate_monthly_account_tier, generate_root_metadata, test_credit_cards,
    };
    use lockbook_core::service::{api_service, test_utils};
    use lockbook_models::api::*;
    use lockbook_models::file_metadata::FileType;
    use rand::RngCore;

    #[test]
    fn switch_to_premium_and_back() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);

        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        api_service::request(
            &account,
            SwitchAccountTierRequest {
                account_tier: generate_monthly_account_tier(
                    test_credit_cards::GOOD,
                    None,
                    None,
                    None,
                ),
            },
        )
        .unwrap();

        api_service::request(
            &account,
            SwitchAccountTierRequest { account_tier: AccountTier::Free },
        )
        .unwrap();
    }

    #[test]
    fn new_tier_is_old_tier() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);
        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        let result = api_service::request(
            &account,
            SwitchAccountTierRequest { account_tier: AccountTier::Free },
        );

        assert_matches!(
            result,
            Err(ApiError::<SwitchAccountTierError>::Endpoint(
                SwitchAccountTierError::NewTierIsOldTier
            ))
        );

        api_service::request(
            &account,
            SwitchAccountTierRequest {
                account_tier: generate_monthly_account_tier(
                    test_credit_cards::GOOD,
                    None,
                    None,
                    None,
                ),
            },
        )
        .unwrap();

        let result = api_service::request(
            &account,
            SwitchAccountTierRequest {
                account_tier: generate_monthly_account_tier(
                    test_credit_cards::GOOD,
                    None,
                    None,
                    None,
                ),
            },
        );

        assert_matches!(
            result,
            Err(ApiError::<SwitchAccountTierError>::Endpoint(
                SwitchAccountTierError::NewTierIsOldTier
            ))
        );
    }

    #[test]
    fn card_does_not_exist() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);
        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        let result = api_service::request(
            &account,
            SwitchAccountTierRequest { account_tier: AccountTier::Premium(PaymentMethod::OldCard) },
        );

        assert_matches!(
            result,
            Err(ApiError::<SwitchAccountTierError>::Endpoint(
                SwitchAccountTierError::OldCardDoesNotExist
            ))
        );
    }

    #[test]
    fn card_decline() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);
        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        let scenarios = vec![
            (test_credit_cards::decline::GENERIC, CardDeclineReason::Generic),
            (test_credit_cards::decline::LOST_CARD, CardDeclineReason::Generic), // core should not be informed a card is stolen or lost
            (
                test_credit_cards::decline::INSUFFICIENT_FUNDS,
                CardDeclineReason::BalanceOrCreditExceeded,
            ),
            (test_credit_cards::decline::PROCESSING_ERROR, CardDeclineReason::TryAgain),
            (test_credit_cards::decline::EXPIRED_CARD, CardDeclineReason::ExpiredCard),
        ];

        for (card_number, expected_err) in scenarios {
            let result = api_service::request(
                &account,
                SwitchAccountTierRequest {
                    account_tier: generate_monthly_account_tier(card_number, None, None, None),
                },
            );

            match result {
                Err(ApiError::<SwitchAccountTierError>::Endpoint(
                    SwitchAccountTierError::CardDeclined(err),
                )) => assert_eq!(err, expected_err),
                other => panic!("expected {:?}, got {:?}", expected_err, other),
            }
        }
    }

    #[test]
    fn invalid_cards() {
        let account = generate_account();
        let (root, _) = generate_root_metadata(&account);
        api_service::request(&account, NewAccountRequest::new(&account, &root)).unwrap();

        let scenarios = vec![
            (test_credit_cards::INVALID_NUMBER, None, None, None, CardRejectReason::Number),
            (test_credit_cards::GOOD, Some(1970), None, None, CardRejectReason::ExpYear),
            (test_credit_cards::GOOD, None, Some(14), None, CardRejectReason::ExpMonth),
            (test_credit_cards::GOOD, None, None, Some("11"), CardRejectReason::CVC),
        ];

        for (card_number, maybe_exp_year, maybe_exp_month, maybe_cvc, expected_err) in scenarios {
            let result = api_service::request(
                &account,
                SwitchAccountTierRequest {
                    account_tier: generate_monthly_account_tier(
                        card_number,
                        maybe_exp_year,
                        maybe_exp_month,
                        maybe_cvc,
                    ),
                },
            );

            match result {
                Err(ApiError::<SwitchAccountTierError>::Endpoint(
                    SwitchAccountTierError::InvalidCreditCard(err),
                )) => assert_eq!(err, expected_err),
                other => panic!("expected {:?}, got {:?}", expected_err, other),
            }
        }
    }

    #[test]
    fn downgrade_denied() {
        let config = test_utils::test_config();
        let (account, root) = test_utils::create_account(&config);

        loop {
            let mut bytes: [u8; 500000] = [0u8; 500000];

            rand::thread_rng().fill_bytes(&mut bytes);

            let file = lockbook_core::create_file(
                &config,
                &uuid::Uuid::new_v4().to_string(),
                root.id,
                FileType::Document,
            )
            .unwrap();

            lockbook_core::write_document(&config, file.id, &bytes).unwrap();

            lockbook_core::sync_all(&config, None).unwrap();

            // TODO: Currently a users data cap isn't enforced by the server. When it is, this code needs to be updated to not violate usage before upgrading.
            if lockbook_core::get_usage(&config)
                .unwrap()
                .server_usage
                .exact
                > 1000000
            {
                break;
            }
        }

        api_service::request(
            &account,
            SwitchAccountTierRequest {
                account_tier: generate_monthly_account_tier(
                    test_credit_cards::GOOD,
                    None,
                    None,
                    None,
                ),
            },
        )
        .unwrap();

        let result = api_service::request(
            &account,
            SwitchAccountTierRequest { account_tier: AccountTier::Free },
        );

        assert_matches!(
            result,
            Err(ApiError::<SwitchAccountTierError>::Endpoint(
                SwitchAccountTierError::CurrentUsageIsMoreThanNewTier
            ))
        );

        let children = lockbook_core::get_children(&config, root.id).unwrap();

        for child in children {
            lockbook_core::delete_file(&config, child.id).unwrap();

            lockbook_core::sync_all(&config, None).unwrap();

            if lockbook_core::get_usage(&config)
                .unwrap()
                .server_usage
                .exact
                < 1000000
            {
                break;
            }
        }

        api_service::request(
            &account,
            SwitchAccountTierRequest { account_tier: AccountTier::Free },
        )
        .unwrap();
    }
}
