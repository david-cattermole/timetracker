use dirs;
use log::{debug, error};
use shellexpand;
use std::path::PathBuf;

/// Search for an existing file in the home directory, config
/// directory and user directory override.
pub fn find_existing_file_path(user_dir_path: Option<String>, file_name: &str) -> Option<PathBuf> {
    if let Some(value) = user_dir_path {
        let value = shellexpand::full(&value).ok()?.into_owned();
        let mut path = PathBuf::new();
        path.push(value);
        path.push(file_name);
        if path.is_file() {
            return Some(path);
        }
    }

    // $XDG_CONFIG_HOME or $HOME/.config (on Linux)
    if let Some(value) = dirs::config_dir() {
        let mut path = PathBuf::new();
        path.push(value);
        path.push(file_name);
        if path.is_file() {
            return Some(path);
        }
    }

    // $HOME (on Linux)
    if let Some(value) = dirs::home_dir() {
        let mut path = PathBuf::new();
        path.push(value);
        path.push(file_name);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

/// Search for an existing default configuration directory.
pub fn find_existing_configuration_directory_path() -> Option<PathBuf> {
    // $XDG_CONFIG_HOME or $HOME/.config (on Linux)
    if let Some(value) = dirs::config_dir() {
        let mut path = PathBuf::new();
        path.push(value);
        if path.is_dir() {
            return Some(path);
        }
    }

    // $HOME (on Linux)
    if let Some(value) = dirs::home_dir() {
        let mut path = PathBuf::new();
        path.push(value);
        if path.is_dir() {
            return Some(path);
        }
    }

    None
}

pub fn construct_file_path(user_dir_path: &Option<String>, file_name: &str) -> Option<PathBuf> {
    if let Some(value) = user_dir_path {
        let value = shellexpand::full(&value).ok()?.into_owned();
        let mut path = PathBuf::new();
        path.push(value);
        path.push(file_name);
        return Some(path);
    };
    None
}

/// Get the full database file path, used to store timetracker data.
pub fn get_database_file_path(
    database_dir: &String,
    database_file_name: &String,
) -> Option<PathBuf> {
    let database_file_path =
        construct_file_path(&Some(database_dir.to_string()), database_file_name);
    match database_file_path {
        Some(ref value) => {
            debug!("Database File Path: {:?}", value);
        }
        None => {
            error!(
                "ERROR: Could not find Database File. Directory: {:?} File Name: {:?}",
                database_dir, database_file_name
            );
        }
    }
    database_file_path
}
