use crate::index_db::get_file_metadata::to_file_metadata;
use lockbook_core::model::api::FileMetadata;
use postgres::Client as PostgresClient;
use tokio_postgres;
use tokio_postgres::error::Error as PostgresError;

#[derive(Debug)]
pub enum Error {
    Postgres(PostgresError),
}

pub fn get_updates(
    client: &mut PostgresClient,
    username: &String,
    metadata_version: &i64,
) -> Result<Vec<FileMetadata>, Error> {
    match client.query(
        "SELECT file_id, file_name, file_path, file_content_version, file_metadata_version, deleted
    FROM files WHERE username = $1 AND file_metadata_version > $2",
        &[&username, &metadata_version],
    ) {
        Ok(rows) => Ok(rows.iter().map(to_file_metadata).collect()),
        Err(err) => Err(Error::Postgres(err)),
    }
}
