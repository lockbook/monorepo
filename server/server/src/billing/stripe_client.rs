use crate::billing::stripe_client::StripeClientError::{CardDeclined, InvalidCreditCard, Other};
use crate::billing::stripe_model::{
    BasicStripeResponse, SetupPaymentIntentStatus, StripeError, StripeErrorType,
    StripeInvoiceResponse, StripeKnownErrorCode, StripeKnownErrorDeclineCode, StripeMaybeContainer,
    StripePaymentMethodResponse, StripeResult, StripeSetupIntentResponse,
    StripeSubscriptionResponse, SubscriptionStatus,
};
use crate::ServerState;
use lockbook_models::api::{CardDeclineReason, CreditCardRejectReason};
use log::error;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Debug;

static STRIPE_ENDPOINT: &str = "https://api.stripe.com/v1";
static PAYMENT_METHODS_ENDPOINT: &str = "/payment_methods";
static DETACH_ENDPOINT: &str = "/detach";
static CUSTOMER_ENDPOINT: &str = "/customers";
static SUBSCRIPTIONS_ENDPOINT: &str = "/subscriptions";
static SETUP_INTENTS_ENDPOINT: &str = "/setup_intents";
static INVOICES_ENDPOINT: &str = "/invoices";

#[derive(Debug)]
pub enum StripeClientError {
    CardDeclined(CardDeclineReason),
    InvalidCreditCard(CreditCardRejectReason),
    Other(String),
}

pub async fn create_customer(
    server_state: &ServerState,
    payment_method_id: &str,
) -> Result<BasicStripeResponse, StripeClientError> {
    let mut create_customer_form = HashMap::new();
    create_customer_form.insert("payment_method", payment_method_id);

    match send_stripe_request::<BasicStripeResponse>(
        server_state,
        format!("{}{}", STRIPE_ENDPOINT, CUSTOMER_ENDPOINT),
        Method::POST,
        Some(create_customer_form),
    )
    .await?
    {
        StripeResult::Ok(resp) => Ok(resp),
        StripeResult::Err(e) => Err(match_stripe_error(&e.error)),
    }
}

pub async fn delete_customer(
    server_state: &ServerState,
    customer_id: &str,
) -> Result<(), StripeClientError> {
    match send_stripe_request::<BasicStripeResponse>(
        server_state,
        format!("{}{}/{}", STRIPE_ENDPOINT, CUSTOMER_ENDPOINT, customer_id),
        Method::DELETE,
        None,
    )
    .await?
    {
        StripeResult::Ok(_) => Ok(()),
        StripeResult::Err(e) => Err(Other(format!(
            "Stripe returned an error whilst deleting an account: {:?}",
            e
        ))),
    }
}

pub async fn create_payment_method(
    server_state: &ServerState,
    card_number: &str,
    card_exp_year: &str,
    card_exp_month: &str,
    card_cvc: &str,
) -> Result<StripePaymentMethodResponse, StripeClientError> {
    let mut payment_method_form = HashMap::new();
    payment_method_form.insert("type", "card");
    payment_method_form.insert("card[number]", card_number);
    payment_method_form.insert("card[exp_year]", card_exp_year);
    payment_method_form.insert("card[exp_month]", card_exp_month);
    payment_method_form.insert("card[cvc]", card_cvc);

    match send_stripe_request::<StripePaymentMethodResponse>(
        server_state,
        format!("{}{}", STRIPE_ENDPOINT, PAYMENT_METHODS_ENDPOINT),
        Method::POST,
        Some(payment_method_form),
    )
    .await?
    {
        StripeResult::Ok(resp) => Ok(resp),
        StripeResult::Err(e) => Err(match_stripe_error(&e.error)),
    }
}

pub async fn create_setup_intent(
    server_state: &ServerState,
    customer_id: &str,
    payment_method_id: &str,
) -> Result<(), StripeClientError> {
    let mut create_setup_intent_form = HashMap::new();
    create_setup_intent_form.insert("customer", customer_id);
    create_setup_intent_form.insert("payment_method", payment_method_id);
    create_setup_intent_form.insert("confirm", "true");

    match send_stripe_request::<StripeSetupIntentResponse>(
        server_state,
        format!("{}{}", STRIPE_ENDPOINT, SETUP_INTENTS_ENDPOINT),
        Method::POST,
        Some(create_setup_intent_form),
    )
    .await?
    {
        StripeResult::Ok(resp) => match resp.status {
            SetupPaymentIntentStatus::Succeeded => Ok(()),
            SetupPaymentIntentStatus::RequiresAction => Err(Other(format!(
                "Verification required for a card (potentially a european user): {:?}",
                resp
            ))),
            SetupPaymentIntentStatus::RequiresPaymentMethod => match resp.last_setup_error {
                None => Err(Other(format!(
                    "Cannot view stripe's setup intent error despite having a related status: {:?}",
                    resp
                ))),
                Some(e) => match match_stripe_error(&e) {
                    Other(e) => {
                        delete_customer(server_state, customer_id).await?;
                        Err(Other(e))
                    }
                    e => Err(e),
                },
            },
        },
        StripeResult::Err(e) => match match_stripe_error(&e.error) {
            Other(e) => {
                delete_customer(server_state, customer_id).await?;
                Err(Other(e))
            }
            e => Err(e),
        },
    }
}

pub async fn create_subscription(
    server_state: &ServerState,
    customer_id: &str,
    payment_method_id: &str,
) -> Result<StripeSubscriptionResponse, StripeClientError> {
    let mut create_subscription_form = HashMap::new();
    create_subscription_form.insert("customer", customer_id);
    create_subscription_form.insert(
        "items[0][price]",
        server_state.config.stripe.premium_price_id.as_str(),
    );
    create_subscription_form.insert("default_payment_method", payment_method_id);
    create_subscription_form.insert("expand[]", "latest_invoice");
    create_subscription_form.insert("expand[]", "latest_invoice.payment_intent");

    match send_stripe_request::<StripeSubscriptionResponse>(
        server_state,
        format!("{}{}", STRIPE_ENDPOINT, SUBSCRIPTIONS_ENDPOINT),
        Method::POST,
        Some(create_subscription_form),
    )
    .await?
    {
        StripeResult::Ok(resp) => match resp.status {
            SubscriptionStatus::Active => Ok(resp),
            SubscriptionStatus::Incomplete => match resp.latest_invoice.payment_intent {
                    StripeMaybeContainer::Unexpected(_) => Err(Other(format!("Cannot retrieve payment intent; {:?}", resp))),
                    StripeMaybeContainer::Expected(ref payment_intent) => match payment_intent.status {
                        SetupPaymentIntentStatus::Succeeded => Err(Other(format!("Unexpected stripe payment intent status: {:?}", resp))),
                        SetupPaymentIntentStatus::RequiresPaymentMethod => match payment_intent.last_payment_error {
                            None => Err(Other(format!("Cannot view stripe's payment intent error despite having a related status: {:?}", resp))),
                            Some(ref e) => Err(match_stripe_error(e))
                        }
                        SetupPaymentIntentStatus::RequiresAction => Err(Other(format!("Cannot verify payment method whilst creating subscription: {:?}", resp))),
                    }
                }
            _ => Err(Other(format!("Unexpected subscription status (considering payment method has already been checked): {:?}", resp)))
        },
        StripeResult::Err(e) => Err(match_stripe_error(&e.error))
    }
}

pub async fn detach_payment_method_from_customer(
    server_state: &ServerState,
    payment_method_id: &str,
) -> Result<(), StripeClientError> {
    match send_stripe_request::<BasicStripeResponse>(
        server_state,
        format!(
            "{}/{}{}",
            STRIPE_ENDPOINT, payment_method_id, DETACH_ENDPOINT
        ),
        Method::POST,
        None,
    )
    .await?
    {
        StripeResult::Ok(_) => Ok(()),
        StripeResult::Err(e) => Err(Other(format!(
            "Stripe returned an error whilst detaching a payment method from a customer: {:?}",
            e
        ))),
    }
}

pub async fn delete_subscription(
    server_state: &ServerState,
    subscription_id: &str,
) -> Result<(), StripeClientError> {
    match send_stripe_request::<BasicStripeResponse>(
        server_state,
        format!(
            "{}{}/{}",
            STRIPE_ENDPOINT, SUBSCRIPTIONS_ENDPOINT, subscription_id
        ),
        Method::DELETE,
        None,
    )
    .await?
    {
        StripeResult::Ok(_) => Ok(()),
        StripeResult::Err(e) => Err(Other(format!(
            "Stripe returned an error whilst deleting a subscription: {:?}",
            e
        ))),
    }
}

pub async fn retrieve_invoice(
    server_state: &ServerState,
    invoice_id: &str,
) -> Result<StripeInvoiceResponse, StripeClientError> {
    let mut retrieve_subscription = HashMap::new();
    retrieve_subscription.insert("expand[]", "subscription");

    match send_stripe_request::<StripeInvoiceResponse>(
        server_state,
        format!("{}{}/{}", STRIPE_ENDPOINT, INVOICES_ENDPOINT, invoice_id),
        Method::GET,
        Some(retrieve_subscription),
    )
    .await?
    {
        StripeResult::Ok(resp) => Ok(resp),
        StripeResult::Err(e) => Err(match_stripe_error(&e.error)),
    }
}

async fn send_stripe_request<U: DeserializeOwned>(
    server_state: &ServerState,
    url: String,
    method: Method,
    maybe_form: Option<HashMap<&str, &str>>,
) -> Result<StripeResult<U>, StripeClientError> {
    let request = server_state
        .stripe_client
        .request(method, &url)
        .basic_auth::<&str, &str>(&server_state.config.stripe.stripe_secret, None);

    if let Some(form) = maybe_form {
        request.form(&form)
    } else {
        request
    }
    .send()
    .await
    .map_err(|e| {
        Other(format!(
            "Cannot make stripe request at '{}' due to reqwest error: {:?}",
            url, e
        ))
    })?
    .json::<StripeResult<U>>()
    .await
    .map_err(|e| Other(format!("Cannot parse stripe request at '{}': {:?}", url, e)))
}

fn match_stripe_error(error: &StripeError) -> StripeClientError {
    if let StripeErrorType::CardError = error.error_type {
        match error.code {
            StripeMaybeContainer::Expected(ref error_code) => match error_code {
                StripeKnownErrorCode::CardDeclineRateLimitExceeded => {
                    CardDeclined(CardDeclineReason::TooManyTries)
                }
                StripeKnownErrorCode::CardDeclined => match error.decline_code {
                    None => {
                        error!("Although stripe error code is `card_declined`, there is no decline code: {:?}", error);

                        CardDeclined(CardDeclineReason::Generic)
                    }
                    Some(ref decline_code) => match decline_code {
                        StripeMaybeContainer::Expected(ref known_decline_code) => {
                            match known_decline_code {
                                // Try again
                                StripeKnownErrorDeclineCode::ApproveWithId
                                | StripeKnownErrorDeclineCode::IssuerNotAvailable
                                | StripeKnownErrorDeclineCode::ProcessingError
                                | StripeKnownErrorDeclineCode::ReenterTransaction
                                | StripeKnownErrorDeclineCode::TryAgainLater => {
                                    CardDeclined(CardDeclineReason::TryAgain)
                                }

                                //Unknown
                                StripeKnownErrorDeclineCode::CallIssuer
                                | StripeKnownErrorDeclineCode::DoNotTryAgain
                                | StripeKnownErrorDeclineCode::DoNotHonor
                                | StripeKnownErrorDeclineCode::NewAccountInformationAvailable
                                | StripeKnownErrorDeclineCode::RestrictedCard
                                | StripeKnownErrorDeclineCode::RevocationOfAllAuthorizations
                                | StripeKnownErrorDeclineCode::RevocationOfAuthorization
                                | StripeKnownErrorDeclineCode::SecurityViolation
                                | StripeKnownErrorDeclineCode::ServiceNotAllowed
                                | StripeKnownErrorDeclineCode::StopPaymentOrder
                                | StripeKnownErrorDeclineCode::TransactionNotAllowed => {
                                    CardDeclined(CardDeclineReason::Unknown)
                                }

                                // Not supported
                                StripeKnownErrorDeclineCode::CardNotSupported
                                | StripeKnownErrorDeclineCode::CurrencyNotSupported => {
                                    CardDeclined(CardDeclineReason::NotSupported)
                                }

                                // Balance or credit exceeded
                                StripeKnownErrorDeclineCode::CardVelocityExceeded
                                | StripeKnownErrorDeclineCode::InsufficientFunds
                                | StripeKnownErrorDeclineCode::WithdrawalCountLimitExceeded => {
                                    CardDeclined(CardDeclineReason::BalanceOrCreditExceeded)
                                }

                                // Expired card
                                StripeKnownErrorDeclineCode::ExpiredCard => {
                                    CardDeclined(CardDeclineReason::ExpiredCard)
                                }

                                // Generic
                                StripeKnownErrorDeclineCode::Fraudulent
                                | StripeKnownErrorDeclineCode::GenericDecline
                                | StripeKnownErrorDeclineCode::LostCard
                                | StripeKnownErrorDeclineCode::MerchantBlacklist
                                | StripeKnownErrorDeclineCode::NoActionTaken
                                | StripeKnownErrorDeclineCode::NotPermitted
                                | StripeKnownErrorDeclineCode::PickupCard
                                | StripeKnownErrorDeclineCode::StolenCard => {
                                    CardDeclined(CardDeclineReason::Generic)
                                }

                                // Incorrect number
                                StripeKnownErrorDeclineCode::IncorrectNumber
                                | StripeKnownErrorDeclineCode::InvalidNumber => {
                                    CardDeclined(CardDeclineReason::IncorrectNumber)
                                }

                                // Incorrect cvc
                                StripeKnownErrorDeclineCode::IncorrectCvc
                                | StripeKnownErrorDeclineCode::InvalidCvc => {
                                    CardDeclined(CardDeclineReason::IncorrectCVC)
                                }

                                // Incorrect expiry month
                                StripeKnownErrorDeclineCode::InvalidExpiryMonth => {
                                    CardDeclined(CardDeclineReason::IncorrectExpiryMonth)
                                }

                                // Incorrect expiry year
                                StripeKnownErrorDeclineCode::InvalidExpiryYear => {
                                    CardDeclined(CardDeclineReason::IncorrectExpiryYear)
                                }
                            }
                        }
                        StripeMaybeContainer::Unexpected(_) => Other(format!(
                            "Unexpected stripe decline error code encountered: {:?}",
                            error
                        )),
                    },
                },
                StripeKnownErrorCode::ExpiredCard => CardDeclined(CardDeclineReason::ExpiredCard),
                StripeKnownErrorCode::IncorrectCvc | StripeKnownErrorCode::InvalidCvc => {
                    InvalidCreditCard(CreditCardRejectReason::CVC)
                }
                StripeKnownErrorCode::IncorrectNumber | StripeKnownErrorCode::InvalidNumber => {
                    InvalidCreditCard(CreditCardRejectReason::Number)
                }
                StripeKnownErrorCode::InsufficientFunds => {
                    CardDeclined(CardDeclineReason::BalanceOrCreditExceeded)
                }
                StripeKnownErrorCode::InvalidExpiryMonth => {
                    InvalidCreditCard(CreditCardRejectReason::ExpMonth)
                }
                StripeKnownErrorCode::InvalidExpiryYear => {
                    InvalidCreditCard(CreditCardRejectReason::ExpYear)
                }
                StripeKnownErrorCode::ProcessingError => CardDeclined(CardDeclineReason::TryAgain),
                StripeKnownErrorCode::SetupIntentAuthenticationFailure
                | StripeKnownErrorCode::PaymentIntentAuthenticationFailure => {
                    Other(format!("Stripe authentication error: {:?}", error))
                }
            },
            StripeMaybeContainer::Unexpected(_) => Other(format!(
                "Unknown stripe error code encountered: {:?}",
                error
            )),
        }
    } else {
        Other(format!(
            "Unexpected stripe error type encountered: {:?}",
            error
        ))
    }
}
