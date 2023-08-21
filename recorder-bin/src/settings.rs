use clap::{Parser, Subcommand};
use config::ConfigError;
use serde_derive::Deserialize;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_recorder_settings;
use timetracker_core::settings::CoreSettings;

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
#[clap(propagate_version = true)]
pub struct CommandArguments {
    #[clap(subcommand)]
    pub command: CommandModes,

    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    pub database_dir: Option<String>,

    /// Override the name of the database file to open.
    #[clap(long, value_parser)]
    pub database_file_name: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum CommandModes {
    /// Start the Recorder
    Start {
        /// Automatically terminate (SIGTERM) existing
        /// timetracker-recorder processes (to ensure only one process
        /// runs at any one time).
        #[clap(long, value_parser, default_value_t = false)]
        terminate_existing_processes: bool,
    },
    /// Stop the recorder
    Stop,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RecorderAppSettings {
    pub core: CoreSettings,
}

impl RecorderAppSettings {
    pub fn new(arguments: &CommandArguments) -> Result<Self, ConfigError> {
        let builder = new_core_settings(
            arguments.database_dir.clone(),
            arguments.database_file_name.clone(),
            true,
        )?;
        let builder = new_recorder_settings(builder)?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
