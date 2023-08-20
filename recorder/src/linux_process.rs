use crate::linux_x11::ProcessID;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use timetracker_core;

pub fn read_process_environment_variables(process_id: ProcessID) -> HashMap<String, String> {
    // NOTE: This function assumes the OS running is Linux.
    let mut path = PathBuf::new();
    let process_id_str: String = format!("{}", process_id);
    path.push("/");
    path.push("proc");
    path.push(process_id_str);
    path.push("environ");

    // println!("Reading: {:?}", path);
    let file_content = read_to_string(&path).unwrap();
    let lines: Vec<&str> = file_content.split('\0').collect();

    let mut map = HashMap::new();
    for mut line in lines {
        line = line.trim();
        if line.len() > 0 {
            let line_split: Vec<&str> = line.splitn(2, '=').collect();
            if line_split.len() == 2 {
                let key = line_split[0].trim().to_string();
                let value = line_split[1].trim().to_string();
                map.insert(key, value);
            }
        }
    }

    map
}

pub fn get_process_id_executable_name(process_id: ProcessID) -> String {
    // NOTE: This function assumes the OS running is Linux.
    let mut path = PathBuf::new();
    let process_id_str: String = format!("{}", process_id);
    path.push("/");
    path.push("proc");
    path.push(process_id_str);
    path.push("cmdline");

    // println!("Reading: {:?}", path);
    let file_content = read_to_string(&path).unwrap();
    let executable =
        timetracker_core::strip_executable_name(&file_content.replace("\0", " ")).to_string();

    executable
}
