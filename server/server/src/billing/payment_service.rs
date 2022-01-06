use libsecp256k1::PublicKey;
use sqlx::{Postgres, Transaction};
use crate::billing::stripe::SetupIntentStatus;
use crate::billing::stripe_client;
use crate::file_index_repo::{FREE_TIER_SIZE, GetLastStripeCreditCardInfoError};
use crate::ServerError::{ClientError, InternalError};
use crate::{file_index_repo, RequestContext, ServerError, ServerState};
use lockbook_models::api::{AccountTier, CardChoice, GetLastRegisteredCreditCardError, GetLastRegisteredCreditCardRequest, GetLastRegisteredCreditCardResponse, SwitchAccountTierError, SwitchAccountTierRequest, SwitchAccountTierResponse};

pub async fn switch_account_tier(
    context: RequestContext<'_, SwitchAccountTierRequest>,
) -> Result<SwitchAccountTierResponse, ServerError<SwitchAccountTierError>> {
    let (request, server_state) = (&context.request, context.server_state);

    let mut transaction = match server_state.index_db_client.begin().await {
        Ok(t) => t,
        Err(e) => {
            return Err(InternalError(format!("Cannot begin transaction: {:?}", e)));
        }
    };

    let data_cap = file_index_repo::get_account_data_cap(&mut transaction, &context.public_key)
        .await
        .map_err(|e| InternalError(format!("Cannot get account data cap in Postgres: {:?}", e)))?;

    match (data_cap as i64, &request.account_tier) {
        (FREE_TIER_SIZE, AccountTier::Monthly(card)) => {
            create_subscription(&context.public_key, server_state, &mut transaction, card).await?;
        }
        (FREE_TIER_SIZE, AccountTier::Free) |
        (_, AccountTier::Monthly(_)) => {
            return Err(ClientError(SwitchAccountTierError::NewTierIsOldTier));
        }
        (_, AccountTier::Free) => {
            let subscription_id = file_index_repo::get_active_stripe_subscription_id(
                &mut transaction,
                &context.public_key,
            )
                .await
                .map_err(|e| {
                    InternalError(format!(
                        "Cannot retrieve stripe subscription in Postgres: {:?}",
                        e
                    ))
                })?;

            stripe_client::delete_subscription(&server_state, &subscription_id).await?;

            file_index_repo::cancel_stripe_subscription(&mut transaction, &subscription_id)
                .await
                .map_err(|e| {
                    InternalError(format!(
                        "Cannot cancel stripe subscription in Postgres: {:?}",
                        e
                    ))
                })?;
        }
    }

    match transaction.commit().await {
        Ok(()) => Ok(SwitchAccountTierResponse {}),
        Err(e) => Err(InternalError(format!("Cannot commit transaction: {:?}", e))),
    }
}

async fn create_subscription(
    public_key: &PublicKey,
    server_state: &ServerState,
    mut transaction: &mut Transaction<'_, Postgres>,
    card: &CardChoice
) -> Result<(), ServerError<SwitchAccountTierError>> {
    let (customer_id, payment_method_id) = match card {
        CardChoice::NewCard {
            number, exp_year, exp_month, cvc
        } => {
            let payment_method_resp = stripe_client::create_payment_method(
                &server_state,
                number,
                exp_year,
                exp_month,
                cvc
            )
                .await?;

            let customer_id = stripe_client::create_customer(&server_state).await?;

            file_index_repo::attach_stripe_customer_id(
                &mut transaction,
                &customer_id,
                &public_key,
            )
                .await
                .map_err(|e| {
                    InternalError(format!(
                        "Couldn't insert payment method into Postgres: {:?}",
                        e
                    ))
                })?;

            file_index_repo::add_stripe_payment_method(
                &mut transaction,
                &payment_method_resp.id,
                &customer_id,
                &payment_method_resp.card.last4,
            )
                .await
                .map_err(|e| InternalError(format!("Couldn't add payment method to Postgres: {:?}", e)))?;

            let setup_intent_status =
                stripe_client::create_setup_intent(&server_state, &customer_id, &payment_method_resp.id).await?;

            match setup_intent_status {
                SetupIntentStatus::Succeeded => {
                    stripe_client::attach_payment_method_to_customer(
                        &server_state,
                        &customer_id,
                        &payment_method_resp.id,
                    )
                        .await?;

                    (customer_id, payment_method_resp.id)
                }
                _ => return Err(InternalError(format!(
                    "Unexpected confirmation of stripe setup intent: {:?}",
                    setup_intent_status
                )))
            }
        }
        CardChoice::OldCard => {
            let old_card = file_index_repo::get_last_stripe_credit_card_info(&mut transaction, &public_key)
                .await
                .map_err(|e| match e {
                    GetLastStripeCreditCardInfoError::NoPaymentInfo => ClientError(SwitchAccountTierError::PreexistingCardDoesNotExist),
                    _ => InternalError(format!("Cannot get stripe payment method info from Postgres: {:?}", e))
                })?;

            let customer_id = file_index_repo::get_stripe_customer_id(&mut transaction, &public_key)
                .await
                .map_err(|e| InternalError(format!("Cannot get stripe customer id from Postgres: {:?}", e)))?;

            (customer_id, old_card.payment_method_id)
        }
    };

    let subscription_id =
        stripe_client::create_subscription(&server_state, &customer_id, &payment_method_id).await?;

    file_index_repo::add_stripe_subscription(
        &mut transaction,
        &customer_id,
        &subscription_id,
    )
        .await
        .map_err(|e| {
            InternalError(format!(
                "Cannot add stripe subscription in Postgres: {:?}",
                e
            ))
        })
}

pub async fn get_last_registered_credit_card(
    context: RequestContext<'_, GetLastRegisteredCreditCardRequest>,
) -> Result<GetLastRegisteredCreditCardResponse, ServerError<GetLastRegisteredCreditCardError>> {
    let mut transaction = match context.server_state.index_db_client.begin().await {
        Ok(t) => t,
        Err(e) => {
            return Err(InternalError(format!("Cannot begin transaction: {:?}", e)));
        }
    };

    let credit_card =
        file_index_repo::get_last_stripe_credit_card_info(&mut transaction, &context.public_key)
            .await
            .map_err(|e| {
                InternalError(format!("Cannot get all stripe credit card infos: {:?}", e))
            })?;

    Ok(GetLastRegisteredCreditCardResponse { credit_card })
}
