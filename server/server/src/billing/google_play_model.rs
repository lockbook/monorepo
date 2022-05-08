use serde::Deserialize;

pub struct GooglePlayUserInfo {
    pub purchase_token: String,
    pub subscription_id: String,
    pub expiration_time: u64
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperNotification {
    pub version: String,
    pub package_name: String,
    pub event_time_millis: u64,
    pub one_time_product_notification: Option<OneTimeProductNotification>,
    pub subscription_notification: Option<SubscriptionNotification>,
    pub test_notification: Option<TestNotification>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionNotification {
    pub version: String,
    pub notification_type: u32,
    pub purchase_token: String,
    pub subscription_id: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneTimeProductNotification {
    pub version: String,
    pub notification_type: u32,
    pub purchase_token: String,
    pub sku: String
}

#[derive(Debug, Deserialize)]
struct TestNotification {
    pub version: String
}
