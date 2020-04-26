use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, fs, io};

use structopt::StructOpt;
use uuid::Uuid;

use lockbook_core::client::{Client, CreateFileRequest, NewAccountError, CreateFileError};
use lockbook_core::model::state::Config;
use lockbook_core::repo::account_repo::AccountRepo;
use lockbook_core::repo::db_provider::DbProvider;
use lockbook_core::service::account_service::{
    AccountCreationError, AccountImportError, AccountService,
};
use lockbook_core::service::auth_service::AuthService;
use lockbook_core::service::file_service::{FileService, NewFileError, UpdateFileError};
use lockbook_core::{Db, DefaultAccountRepo, DefaultAccountService, DefaultAuthService, DefaultClient, DefaultDbProvider, DefaultFileService, DefaultFileMetadataRepo};
use lockbook_core::model::file_metadata::{FileMetadata, Status};
use lockbook_core::repo::file_metadata_repo::FileMetadataRepo;

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "A secure and intuitive notebook.")]
enum Lockbook {
    /// Create a new file
    New,

    /// Get updates, push changes
    Sync,

    /// Search and edit a file
    Edit,

    /// Browse your files interactively
    Browse,

    /// Search and delete a file
    Remove,

    /// Rename a file
    Move,

    /// Search for a file and see file metadata
    Find,

    /// List all your files
    List,

    /// Bring a file from your computer into Lockbook
    Copy,

    /// Share a file with a collaborator
    Share,

    /// Create a new Lockbook account
    Init,

    /// Import an existing Lockbook
    Import,

    /// See Lockbook's current status
    Status,

    /// Delete the Lockbook data directory from this device
    Nuke,
}

fn main() {
    let args: Lockbook = Lockbook::from_args();
    match args {
        Lockbook::New => new(),
        Lockbook::Sync => unimplemented!(),
        Lockbook::Edit => unimplemented!(),
        Lockbook::Browse => unimplemented!(),
        Lockbook::Remove => unimplemented!(),
        Lockbook::Move => unimplemented!(),
        Lockbook::Find => unimplemented!(),
        Lockbook::List => unimplemented!(),
        Lockbook::Copy => unimplemented!(),
        Lockbook::Share => unimplemented!(),
        Lockbook::Init => init(),
        Lockbook::Import => import(),
        Lockbook::Status => unimplemented!(),
        Lockbook::Nuke => unimplemented!(),
    }
}

fn connect_to_db() -> Db {
    // Save data in LOCKBOOK_CLI_LOCATION or ~/.lockbook/
    let path = env::var("LOCKBOOK_CLI_LOCATION")
        .unwrap_or(format!("{}/.lockbook", env::var("HOME")
            .expect("Could not read env var LOCKBOOK_CLI_LOCATION or HOME, don't know where to place your .lockbook folder"))
        );

    DefaultDbProvider::connect_to_db(&Config {
        writeable_path: path.clone(),
    })
        .expect(&format!("Could not connect to db at path: {}", path))
}

fn get_editor() -> String {
    env::var("VISUAL").unwrap_or(env::var("EDITOR").unwrap_or("vi".to_string()))
}

fn init() {
    let db = connect_to_db();

    print!("Enter a Username: ");
    io::stdout().flush().unwrap();

    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read from stdin");
    username.retain(|c| !c.is_whitespace());

    match DefaultAccountService::create_account(&db, &username) {
        Ok(_) => println!("Account created successfully!"),
        Err(err) => match err {
            AccountCreationError::KeyGenerationError(e) => {
                eprintln!("Could not generate keypair, error: {}", e)
            }

            AccountCreationError::PersistenceError(_) => {
                eprintln!("Could not persist data, error: ")
            }

            AccountCreationError::ApiError(api_err) => match api_err {
                NewAccountError::SendFailed(_) => eprintln!("Network Error Occurred"),
                NewAccountError::UsernameTaken => {
                    eprintln!("Username {} not available!", &username)
                }
                _ => eprintln!("Unknown Error Occurred!"),
            },

            AccountCreationError::AuthGenFailure(_) => {
                eprintln!("Could not use private key to sign message")
            }

            AccountCreationError::KeySerializationError(_) => eprintln!("Could not serialize key"),
        },
    }
}

fn import() {
    let db = connect_to_db();
    println!("To import an existing Lockbook, enter an Account Export String:");

    let mut account_string = String::new();
    io::stdin()
        .read_line(&mut account_string)
        .expect("Failed to read from stdin");

    match DefaultAccountService::import_account(&db, &account_string) {
        Ok(_) => println!("Account imported successfully!"),
        Err(err) => match err {
            AccountImportError::AccountStringCorrupted(_) => eprintln!("Account String corrupted!"),
            AccountImportError::PersistenceError(_) => eprintln!("Could not persist data!"),
        },
    }
}

fn new() {
    let db = connect_to_db();
    let account =
        DefaultAccountRepo::get_account(&db).expect("No account found, run init, import or help.");

    let file_location = format!("/tmp/{}", Uuid::new_v4().to_string());
    let temp_file_path = Path::new(file_location.as_str());
    File::create(&temp_file_path)
        .expect(format!("Could not create temporary file: {}", &file_location).as_str());

    print!("Enter a filename: ");
    io::stdout().flush().unwrap();

    let mut file_name = String::new();
    io::stdin()
        .read_line(&mut file_name)
        .expect("Failed to read from stdin");
    println!("Creating file {}", &file_name);

    let file_metadata = match DefaultFileService::create(&db, &file_name, &file_location) {
        Ok(file_metadata) => file_metadata,
        Err(error) => match error {
            NewFileError::AccountRetrievalError(_) => {
                panic!("No account found, run init, import, or help.")
            }
            NewFileError::EncryptedFileError(_) => panic!("Failed to perform encryption!"),
            NewFileError::SavingMetadataFailed(_) => {
                panic!("Failed to persist file metadata locally")
            }
            NewFileError::SavingFileContentsFailed(_) => {
                panic!("Failed to persist file contents locally")
            }
        },
    };

    let edit_was_successful = Command::new(get_editor())
        .arg(&file_location)
        .spawn()
        .expect(
            format!(
                "Failed to spawn: {}, content location: {}",
                get_editor(),
                &file_location
            )
                .as_str(),
        )
        .wait()
        .expect(
            format!(
                "Failed to wait for spawned process: {}, content location: {}",
                get_editor(),
                &file_location
            )
                .as_str(),
        )
        .success();

    if edit_was_successful {
        let file_content =
            fs::read_to_string(temp_file_path).expect("Could not read file that was edited");

        let encrypted_file = match DefaultFileService::update(&db, &file_metadata.id, &file_content)
        {
            Ok(file) => file,
            Err(err) => match err {
                UpdateFileError::AccountRetrievalError(_) => panic!(
                    "No account found, run init, import, or help, aborting without cleaning up"
                ),
                UpdateFileError::FileRetrievalError(_) => {
                    panic!("Failed to get file being edited, aborting without cleaning up")
                }
                UpdateFileError::EncryptedWriteError(_) => {
                    panic!("Failed to perform encryption!, aborting without cleaning up")
                }
                UpdateFileError::MetadataDbError(_) => {
                    panic!("Failed to update file metadata, aborting without cleaning up")
                }
            },
        };

        match DefaultClient::create_file(&CreateFileRequest {
            username: account.username.clone(),
            auth: DefaultAuthService::generate_auth(&account).expect("Failed to sign message"),
            file_id: file_metadata.id.clone(),
            file_name: file_metadata.name.clone(),
            file_path: file_location.clone(),
            file_content: serde_json::to_string(&encrypted_file)
                .expect("Failed to serialize encrypted file"),
        }) {
            Ok(version) => {
                DefaultFileMetadataRepo::update(&db, &FileMetadata {
                    id: file_metadata.id,
                    name: file_metadata.name,
                    path: file_metadata.path,
                    updated_at: 0,
                    version,
                    status: Status::Synced,
                }).expect("Failed to update metadata repo");
                print!("File saved locally and synced!")
            }
            Err(err) => match err {
                CreateFileError::SendFailed(_) => eprintln!("Network error occurred, file will be sent next sync"),
                _ => eprint!("Unknown error occurred sending file, file exists locally.")
            }
        }
    } else {
        eprintln!(
            "{} indicated a problem, aborting and cleaning up",
            get_editor()
        );
    }

    fs::remove_file(&temp_file_path)
        .expect(format!("Failed to delete temporary file: {}", &file_location).as_str());
}
