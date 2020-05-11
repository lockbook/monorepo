use crate::index_db;
use lockbook_core::model::api::{NewAccountError, NewAccountRequest, NewAccountResponse};

pub fn new_account(
    index_db_client: &mut postgres::Client,
    request: NewAccountRequest,
) -> Result<NewAccountResponse, NewAccountError> {
    let new_account_result =
        index_db::new_account(index_db_client, &request.username, &request.public_key);
    match new_account_result {
        Ok(()) => Ok(NewAccountResponse {}),
        Err(index_db::new_account::Error::UsernameTaken) => Err(NewAccountError::UsernameTaken),
        Err(index_db::new_account::Error::Uninterpreted(_)) => {
            println!("Internal server error! {:?}", new_account_result);
            Err(NewAccountError::InternalError)
        }
    }
}
