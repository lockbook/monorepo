use crate::service::api_service;
use crate::{account_repo, Config, CoreError};
use lockbook_models::api::{AccountTier, CreditCardInfo, GetRegisteredCreditCardsRequest, RegisterCreditCardError, RegisterCreditCardRequest, RegisterCreditCardResponse, RemoveCreditCardError, RemoveCreditCardRequest, SwitchAccountTierError, SwitchAccountTierRequest};
use crate::model::errors::core_err_unexpected;
use crate::service::api_service::ApiError;

pub fn add_credit_card(
    config: &Config,
    card_number: String,
    exp_month: String,
    exp_year: String,
    cvc: String,
) -> Result<CreditCardInfo, CoreError> {
    let account = account_repo::get(config)?;

    match api_service::request(
        &account,
        RegisterCreditCardRequest {
            card_number,
            exp_month,
            exp_year,
            cvc,
        },
    ) {
        Ok(response) => Ok(response.credit_card_info),
        Err(ApiError::Endpoint(RegisterCreditCardError::InvalidCreditCardFormat)) => Err(CoreError::InvalidCreditCard),
        Err(ApiError::SendFailed(_)) => Err(CoreError::ServerUnreachable),
        Err(e) => Err(core_err_unexpected(e))
    }
}

pub fn switch_account_tier(
    config: &Config,
    new_account_tier: AccountTier
) -> Result<(), CoreError> {
    let account = account_repo::get(config)?;

    api_service::request(
        &account,
        SwitchAccountTierRequest {
            account_tier: new_account_tier
        },
    ).map_err(|e| match e {
        Err(ApiError::Endpoint(SwitchAccountTierError::PaymentMethodDoesNotExist)) => CoreError::PaymentMethodDoesNotExist,
        Err(ApiError::Endpoint(SwitchAccountTierError::NewTierIsOldTier)) => CoreError::NewTierIsOldTier,
        Err(ApiError::SendFailed(_)) => CoreError::ServerUnreachable,
        Err(e) => core_err_unexpected(e)
    })
}

pub fn remove_credit_card(
    config: &Config,
    payment_method_id: String
) -> Result<(), CoreError> {
    let account = account_repo::get(config)?;

    api_service::request(
        &account,
        RemoveCreditCardRequest {
            payment_method_id
        },
    ).map_err(|e| match e {
        Err(ApiError::Endpoint(RemoveCreditCardError::PaymentMethodDoesNotExist)) => CoreError::PaymentMethodDoesNotExist,
        Err(ApiError::SendFailed(_)) => CoreError::ServerUnreachable,
        Err(e) => core_err_unexpected(e)
    })
}

pub fn get_registered_credit_cards(
    config: &Config,
) -> Result<List<CreditCardInfo>, CoreError> {
    let account = account_repo::get(config)?;

    api_service::request(
        &account,
        GetRegisteredCreditCardsRequest {},
    ).map_err(CoreError::from).map(|response| response.credit_card_infos)
}
