use clap::Parser;
use config::ConfigError;
use serde_derive::{Deserialize, Serialize};
use timetracker_core::filesystem::find_existing_configuration_directory_path;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_print_settings;
use timetracker_core::settings::validate_core_settings;
use timetracker_core::settings::CoreSettings;
use timetracker_core::settings::PrintSettings;
use timetracker_core::settings::DEFAULT_CONFIG_FILE_NAME;

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
pub struct CommandArguments {
    /// If true, ignore any user configuration files and return
    /// default configuration options.
    #[clap(long, value_parser, default_value_t = false)]
    pub defaults: bool,

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
        let mut builder = new_core_settings(None, None, arguments.defaults)?;

        let default_config_dir = find_existing_configuration_directory_path()
            .expect("Could not find a default config directory ($HOME, $HOME/.config or $XDG_CONFIG_HOME).")
        .into_os_string()
        .into_string()
        .unwrap();

        builder = builder
            .set_default("configure.config_dir", default_config_dir)?
            .set_default("configure.config_file_name", DEFAULT_CONFIG_FILE_NAME)?;

        let settings: Self = builder.build()?.try_deserialize()?;
        validate_core_settings(&settings.core).unwrap();

        Ok(settings)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullConfigurationSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl FullConfigurationSettings {
    pub fn new(defaults: bool) -> Result<Self, ConfigError> {
        let mut builder = new_core_settings(None, None, defaults)?;
        builder = new_print_settings(builder)?;

        let settings: Self = builder.build()?.try_deserialize()?;
        validate_core_settings(&settings.core).unwrap();

        Ok(settings)
    }
}
