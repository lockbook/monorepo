use std::path::PathBuf;

use structopt::StructOpt;

use lockbook_core::init_logger_safely;
use lockbook_core::repo::file_metadata_repo::Filter::{DocumentsOnly, LeafNodesOnly};

mod copy;
mod edit;
mod export;
mod import;
mod init;
mod list;
mod new;
mod print;
mod remove;
mod status;
mod sync;
mod utils;
mod whoami;

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "A secure and intuitive notebook.")]
enum Lockbook {
    /// Create a new file
    New,

    /// Get updates, push changes
    Sync,

    /// Search and edit a file
    Edit,

    /// Search and delete a file
    Remove,

    /// List all your files
    List,

    /// List all your files
    #[structopt(name = "list-docs")]
    ListDocs,

    /// List all your files
    #[structopt(name = "list-all")]
    ListAll,

    /// Bring a file from your computer into Lockbook
    Copy { file: PathBuf },

    /// Create a new Lockbook account
    Init,

    /// Import an existing Lockbook
    Import,

    /// What operations a sync would perform
    Status,

    /// Export your private key
    Export,

    /// Print the contents of a file
    Print,

    /// Display lockbook username
    #[structopt(name = "whoami")]
    WhoAmI,
}

fn main() {
    init_logger_safely();
    let args: Lockbook = Lockbook::from_args();
    match args {
        Lockbook::New => new::new(),
        Lockbook::Sync => sync::sync(),
        Lockbook::Edit => edit::edit(),
        Lockbook::Remove => remove::remove(),
        Lockbook::List => list::list(Some(LeafNodesOnly)),
        Lockbook::ListAll => list::list(None),
        Lockbook::ListDocs => list::list(Some(DocumentsOnly)),
        Lockbook::Init => init::init(),
        Lockbook::Import => import::import(),
        Lockbook::Status => status::status(),
        Lockbook::Export => export::export(),
        Lockbook::WhoAmI => whoami::whoami(),
        Lockbook::Print => print::print(),
        Lockbook::Copy { file: path } => copy::copy(path),
    }
}
