use crate::linux_x11::ProcessID;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::Command;
use timetracker_core::format_short_executable_name;

type UserID = u32;

pub fn read_process_environment_variables(
    process_id: ProcessID,
) -> Result<HashMap<String, String>> {
    // NOTE: This function assumes the OS running is Linux.

    let process_id_str: String = format!("{}", process_id);

    let mut path = PathBuf::new();
    path.push("/");
    path.push("proc");
    path.push(process_id_str);
    path.push("environ");

    // println!("Reading: {:?}", path);
    let file_content = read_to_string(&path)?;
    let lines: Vec<&str> = file_content.split('\0').collect();

    let mut map = HashMap::new();
    for mut line in lines {
        line = line.trim();
        if !line.is_empty() {
            let line_split: Vec<&str> = line.splitn(2, '=').collect();
            if line_split.len() == 2 {
                let key = line_split[0].trim().to_string();
                let value = line_split[1].trim().to_string();
                map.insert(key, value);
            }
        }
    }

    Ok(map)
}

fn parse_loginuid_file_contents(file_content: &String) -> Result<UserID> {
    let lines: Vec<&str> = file_content.split('\0').collect();

    match lines.is_empty() {
        true => Err(anyhow!(
            "/proc/####/loginuid file does not have any lines in it."
        )),
        false => {
            let line = lines[0].trim();
            let user_id = line.parse::<UserID>()?;
            Ok(user_id)
        }
    }
}

/// Get the current user's user id (uid).
///
/// On Linux multiple users may be logged into the same machine and
/// running timetracker-recorder at the same time.
pub fn get_user_id_running_process_id(process_id: ProcessID) -> Result<UserID> {
    // NOTE: This function assumes the OS running is Linux.

    // TODO: Make sure this function actually returns the correct user
    // ID when there are multiple users on a computer running this
    // process concurrently.

    let process_id_str: String = format!("{}", process_id);

    let mut path = PathBuf::new();
    path.push("/");
    path.push("proc");
    path.push(process_id_str);
    path.push("loginuid");

    let file_content = read_to_string(&path)?;
    let user_id = parse_loginuid_file_contents(&file_content)?;
    Ok(user_id)
}

pub fn find_process_ids_by_user_and_executable_name(
    executable_name: &str,
    this_user_id: UserID,
    this_process_id: ProcessID,
) -> Result<Vec<ProcessID>> {
    // NOTE: This function assumes the OS running is Linux.

    // TODO: Get the current user's user id (uid) and make sure to
    // process the process ids to only processes that are owned by the
    // current user. On Linux multiple users may be logged into the
    // same machine and running timetracker-recorder at the same time
    // on the same machine.
    let mut path = PathBuf::new();
    path.push("/");
    path.push("proc");

    let read_directory = std::fs::read_dir(path)?;
    let valid_directories: Vec<_> = read_directory
        .filter_map(|entry| {
            let entry = entry.ok()?.path();

            if entry.is_dir() {
                Some(entry)
            } else {
                None
            }
        })
        .collect();

    let process_ids: Vec<ProcessID> = valid_directories
        .iter()
        .filter_map(|p| {
            // debug!("Reading: {:?}", p);
            let process_id_str = p.file_name();

            let mut loginuid_path = p.to_path_buf();
            loginuid_path.push("loginuid");
            let file_content = read_to_string(loginuid_path).ok()?;
            let user_id = parse_loginuid_file_contents(&file_content).ok()?;
            if user_id != this_user_id {
                return None;
            }

            let mut cmdline_path = p.to_path_buf();
            cmdline_path.push("cmdline");
            let file_content = read_to_string(cmdline_path).ok()?;

            let executable =
                timetracker_core::strip_executable_name(&file_content.replace('\0', " "))
                    .to_string();
            let executable_short = format_short_executable_name(&executable);

            if executable_name == executable_short {
                match process_id_str {
                    Some(value) => {
                        let process_id = value
                            .to_os_string()
                            .into_string()
                            .ok()?
                            .parse::<ProcessID>()
                            .ok()?;
                        if this_process_id != process_id {
                            Some(process_id)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            } else {
                None
            }
        })
        .collect();

    Ok(process_ids)
}

pub fn terminate_processes(process_ids: &Vec<ProcessID>) -> Result<()> {
    for process_id in process_ids {
        let mut kill = Command::new("kill")
            .args(["-s", "SIGTERM", &process_id.to_string()])
            .spawn()?;
        kill.wait()?;
    }
    Ok(())
}

pub fn get_process_id_executable_name(process_id: ProcessID) -> Result<String> {
    // NOTE: This function assumes the OS running is Linux.
    let mut path = PathBuf::new();
    let process_id_str: String = format!("{}", process_id);
    path.push("/");
    path.push("proc");
    path.push(process_id_str);
    path.push("cmdline");

    // println!("Reading: {:?}", path);
    let file_content = read_to_string(&path)?;
    let executable =
        timetracker_core::strip_executable_name(&file_content.replace('\0', " ")).to_string();

    Ok(executable)
}
