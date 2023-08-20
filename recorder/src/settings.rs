use clap::Parser;
use config::ConfigError;
use serde_derive::Deserialize;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_recorder_settings;
use timetracker_core::settings::CoreSettings;

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
pub struct CommandArguments {
    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    database_dir: Option<String>,

    /// Override the name of the database file to open.
    #[clap(long, value_parser)]
    database_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RecorderAppSettings {
    pub core: CoreSettings,
}

impl RecorderAppSettings {
    pub fn new(arguments: CommandArguments) -> Result<Self, ConfigError> {
        let builder =
            new_core_settings(arguments.database_dir, arguments.database_file_name, true)?;
        let builder = new_recorder_settings(builder)?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
