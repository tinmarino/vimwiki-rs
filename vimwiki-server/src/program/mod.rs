mod server;
mod stdin;
mod watcher;
use watcher::*;

use crate::{config::Mode, database, Config};
use derive_more::{Display, From};
use entity::DatabaseRc;

/// Alias for a result with a program error
pub type ProgramResult<T, E = ProgramError> = std::result::Result<T, E>;

#[derive(Debug, Display, From)]
pub enum ProgramError {
    #[display(fmt = "Could not load database: {:?}", _0)]
    LoadDatabase(async_graphql::Error),
    #[display(fmt = "Could not start file watcher: {}", _0)]
    FileWatcher(notify::Error),
}

pub struct Program;

impl Program {
    /// Runs our program
    pub async fn run(config: Config) -> ProgramResult<()> {
        // Load our database using the provided configuration and any
        // cached data from a previous run
        let database =
            database::load(&config).await.map_err(ProgramError::from)?;

        // Initialize our watcher to update the database based on changes
        // that occur in wikis and standalone files
        let _watcher =
            Watcher::initialize(&config, DatabaseRc::clone(&database))
                .await
                .map_err(ProgramError::from)?;

        match config.mode {
            Mode::Stdin => stdin::run(config).await,
            Mode::Http => server::run(config).await,
        }

        Ok(())
    }
}
