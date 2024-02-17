#[macro_use]
extern crate num_derive;

use log::debug;
use terminfo;

pub mod entries;
pub mod filesystem;
pub mod format;
pub mod settings;
pub mod storage;

/// Removes flags from the executable command name. Only the
/// executable file path should be retained.
pub fn strip_executable_name(name: &str) -> &str {
    // Assumes a 'name' such as:
    // "/path/to/exe/exe_file --flag /path/to/file_path.jpg".

    // Strips off end of string, at first space character:
    // "/path/to/exe/exe_file --flag /path/to/file_path.jpg" to "/path/to/exe/file"
    match name.find(' ') {
        Some(end_index) => &name[..end_index],
        None => name,
    }
}

pub fn format_short_executable_name(name: &str) -> &str {
    // Assumes a 'name' such as:
    // "/path/to/exe/exe_file --flag /path/to/file_path.jpg".

    // Strips off end of string, at first space character:
    // "/path/to/exe/exe_file --flag /path/to/file_path.jpg" to "/path/to/exe/file"
    let strip_end = strip_executable_name(name);

    // Strips off start of string, at last forward-slash character:
    // "/path/to/exe/exe_file" to "exe_file"
    match strip_end.rfind('/') {
        Some(start_index) => &strip_end[start_index + 1..],
        None => strip_end,
    }
}

pub fn terminal_supports_color() -> bool {
    let info = terminfo::Database::from_env().unwrap();
    let terminal_max_colors = info.get::<terminfo::capability::MaxColors>();
    debug!("terminal_max_colors={:?}", terminal_max_colors);
    let color_is_supported = match terminal_max_colors {
        Some(n) => n.0 > 0,
        None => false,
    };
    debug!("terminal_supports_color={}", color_is_supported);
    color_is_supported
}
