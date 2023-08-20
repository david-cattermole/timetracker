use crate::filesystem::find_existing_file_path;
use config::{
    builder::DefaultState, Config, ConfigBuilder, ConfigError, Environment, File, FileFormat,
};
use serde_derive::Deserialize;
use std::collections::HashMap;

/// How often will the recorder query the system to find data?
pub const RECORD_INTERVAL_SECONDS: u64 = 1;

/// How many seconds does the user need to be idle before we consider
/// the user to be in an idle state?
pub const USER_IS_IDLE_LIMIT_SECONDS: u64 = 30;

/// The name of the file used to save timetracker data.
const DATABASE_FILE_NAME: &str = ".timetracker.sqlite3";

/// The database default directory - the home directory. This path is
/// expanded using the 'shellexpand' crate.
const DATABASE_DIR: &str = "~/";

/// The name of the file used to read timetracker configuration data.
///
/// The configuration file is found by searching in the
/// "TIMETRACKER_CONFIG_PATH" environment variable (if it exists),
/// then in the home directory.
const CONFIG_FILE_NAME: &str = ".timetracker.toml";

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct EnvVarSettings {
    pub names: Vec<String>,
    pub labels: HashMap<String, String>,
    pub ordering: HashMap<String, i32>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct CoreSettings {
    pub database_dir: String,
    pub database_file_name: String,
    pub record_interval_seconds: u64,
    pub user_is_idle_limit_seconds: u64,
    pub environment_variables: EnvVarSettings,
}

pub fn new_core_settings(
    database_dir: Option<String>,
    database_file_name: Option<String>,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    let mut env_var_names = Vec::new();
    env_var_names.push("project".to_string());
    env_var_names.push("sequence".to_string());
    env_var_names.push("shot".to_string());

    let env_var_ordering = HashMap::<String, i32>::new();
    let env_var_labels = HashMap::<String, String>::new();

    let mut builder = Config::builder()
        .set_default("core.record_interval_seconds", RECORD_INTERVAL_SECONDS)?
        .set_default(
            "core.user_is_idle_limit_seconds",
            USER_IS_IDLE_LIMIT_SECONDS,
        )?
        .set_default("core.database_dir", DATABASE_DIR)?
        .set_default("core.database_file_name", DATABASE_FILE_NAME)?
        .set_default("core.environment_variables.names", env_var_names)?
        .set_default("core.environment_variables.ordering", env_var_ordering)?
        .set_default("core.environment_variables.labels", env_var_labels)?
        // Allows settings from environment variables (with a prefix
        // of TIMETRACKER) eg `TIMETRACKER_CORE_DATABASE_DIR=1 ./target/app` to
        // set the `core.database_dir` key.
        .add_source(Environment::with_prefix("timetracker"))
        // Overrides
        .set_override_option("core.database_dir", database_dir)?
        .set_override_option("core.database_file_name", database_file_name)?;

    // Runtime configuration file options.
    let config_file_name = CONFIG_FILE_NAME;
    let env_config_path = std::env::var("TIMETRACKER_CONFIG_PATH");
    let user_config_path: Option<String> = match env_config_path {
        Ok(value) => Some(value),
        Err(..) => None,
    };
    let config_file_path = find_existing_file_path(user_config_path, config_file_name);
    if let Some(file_path) = config_file_path {
        if let Some(file_path) = file_path.to_str() {
            builder = builder.add_source(File::new(file_path, FileFormat::Toml).required(false));
        }
    }

    Result::Ok(builder)
}
