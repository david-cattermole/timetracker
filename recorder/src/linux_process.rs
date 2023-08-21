use crate::linux_x11::ProcessID;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::Command;
use timetracker_core::format_short_executable_name;

pub fn read_process_environment_variables(
    process_id: ProcessID,
) -> Result<HashMap<String, String>> {
    // NOTE: This function assumes the OS running is Linux.
    let mut path = PathBuf::new();
    let process_id_str: String = format!("{}", process_id);
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

pub fn find_process_ids_by_executable_name(
    executable_name: &str,
    this_process_id: u32,
) -> Result<Vec<ProcessID>> {
    // NOTE: This function assumes the OS running is Linux.
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
            let mut path = p.to_path_buf();
            path.push("cmdline");
            let file_content = read_to_string(path).ok()?;

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
