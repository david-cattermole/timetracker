use clap::Parser;
use config::ConfigError;
use serde_derive::{Deserialize, Serialize};
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_print_settings;
use timetracker_core::settings::CoreSettings;
use timetracker_core::settings::PrintSettings;
use timetracker_core::settings::CONFIG_DIR;
use timetracker_core::settings::CONFIG_FILE_NAME;

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
pub struct CommandArguments {
    /// Load the existing user setting values?
    #[clap(long, value_parser, default_value_t = false)]
    pub load_user_overrides: bool,

    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    pub config_dir: Option<String>,

    /// Override the name of the configuration file.
    #[clap(long, value_parser)]
    pub config_file_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct ConfigureSettings {
    pub config_dir: String,
    pub config_file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct ConfigureAppSettings {
    pub core: CoreSettings,
    pub configure: ConfigureSettings,
}

impl ConfigureAppSettings {
    pub fn new(arguments: &CommandArguments) -> Result<Self, ConfigError> {
        let mut builder = new_core_settings(None, None, arguments.load_user_overrides)?;

        builder = builder
            .set_default("configure.config_dir", CONFIG_DIR)?
            .set_default("configure.config_file_name", CONFIG_FILE_NAME)?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullConfigurationSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl FullConfigurationSettings {
    pub fn new(load_user_overrides: bool) -> Result<Self, ConfigError> {
        let mut builder = new_core_settings(None, None, load_user_overrides)?;
        builder = new_print_settings(builder)?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
