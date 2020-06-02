use crate::files_db;
use crate::index_db;
use crate::services::username_is_valid;
use crate::ServerState;
use lockbook_core::model::api::{DeleteFileError, DeleteFileRequest, DeleteFileResponse};

pub async fn handle(
    server_state: &mut ServerState,
    request: DeleteFileRequest,
) -> Result<DeleteFileResponse, DeleteFileError> {
    if !username_is_valid(&request.username) {
        return Err(DeleteFileError::InvalidUsername);
    }
    let transaction = match server_state.index_db_client.transaction().await {
        Ok(t) => t,
        Err(e) => {
            error!("Internal server error! Cannot begin transaction: {:?}", e);
            return Err(DeleteFileError::InternalError);
        }
    };
    let index_db_delete_file_result = index_db::delete_file(&transaction, &request.file_id).await;
    match index_db_delete_file_result {
        Ok(_) => {}
        Err(index_db::delete_file::Error::FileDoesNotExist) => {
            return Err(DeleteFileError::FileNotFound)
        }
        Err(index_db::delete_file::Error::FileDeleted) => return Err(DeleteFileError::FileDeleted),
        Err(index_db::delete_file::Error::Uninterpreted(_)) => {
            error!("Internal server error! {:?}", index_db_delete_file_result);
            return Err(DeleteFileError::InternalError);
        }
        Err(index_db::delete_file::Error::VersionGeneration(_)) => {
            error!("Internal server error! {:?}", index_db_delete_file_result);
            return Err(DeleteFileError::InternalError);
        }
    };

    let filed_db_delete_file_result =
        files_db::delete_file(&server_state.files_db_client, &request.file_id).await;
    let result = match filed_db_delete_file_result {
        Ok(()) => Ok(DeleteFileResponse {}),
        Err(_) => {
            error!("Internal server error! {:?}", filed_db_delete_file_result);
            Err(DeleteFileError::InternalError)
        }
    };

    match transaction.commit().await {
        Ok(_) => result,
        Err(e) => {
            error!("Internal server error! Cannot commit transaction: {:?}", e);
            Err(DeleteFileError::InternalError)
        }
    }
}
