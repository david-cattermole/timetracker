use crate::linux_process::get_process_id_executable_name;
use crate::linux_process::read_process_environment_variables;
use crate::settings::CommandArguments;
use crate::settings::RecorderAppSettings;
use anyhow::{bail, Result};
use chrono;
use clap::Parser;
use log::{debug, error};
use timetracker_core::entries::Entry;
use timetracker_core::entries::EntryStatus;
use timetracker_core::entries::EntryVariablesList;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::storage::Storage;

mod linux_process;
mod linux_x11;
mod settings;

/// How many enties are stored in memory before being saved to the
/// storage.
const ENTRY_BUFFER_MAX_COUNT: usize = 10;

/// The global buffer of entries stored in memory, waiting to be
/// written to storage.
static mut ENTRY_BUFFER: Vec<Entry> = Vec::new();

/// The global status of the user; Is the user active or idle?
static mut ENTRY_STATUS: EntryStatus = EntryStatus::Uninitialized;

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter("TIMETRACKER_LOG")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = RecorderAppSettings::new(args);
    if !settings.is_ok() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings.unwrap();
    debug!("Settings validated: {:#?}", settings);

    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    );

    gtk::init()?;

    let interval_seconds = settings.core.record_interval_seconds.try_into()?;
    let _source_id = glib::source::timeout_add_seconds_local(interval_seconds, move || {
        let idle_time_sec = linux_x11::get_user_idle_time_from_x11();
        if idle_time_sec > settings.core.user_is_idle_limit_seconds {
            unsafe {
                ENTRY_STATUS = EntryStatus::Idle;
            }
        } else {
            unsafe {
                ENTRY_STATUS = EntryStatus::Active;
            }
        }

        let mut env_var_list = EntryVariablesList::empty();
        let name_count = settings.core.environment_variables.names.len();
        if name_count > 0 {
            env_var_list.var1_name = Some(settings.core.environment_variables.names[0].clone());
        }
        if name_count > 1 {
            env_var_list.var2_name = Some(settings.core.environment_variables.names[1].clone());
        }
        if name_count > 2 {
            env_var_list.var3_name = Some(settings.core.environment_variables.names[2].clone());
        }
        if name_count > 3 {
            env_var_list.var4_name = Some(settings.core.environment_variables.names[3].clone());
        }

        let process_id = linux_x11::get_active_window_process_id_from_x11();
        debug!("Process ID: {:?}", process_id);
        match process_id {
            0 => (),
            _ => {
                let environ_vars = read_process_environment_variables(process_id);
                env_var_list.replace_with_environ_vars(&environ_vars);
                env_var_list.executable =
                    Some(get_process_id_executable_name(process_id).to_owned())
            }
        };

        let now_seconds = chrono::Utc::now().timestamp() as u64;
        debug!("Time: {:?}", now_seconds);

        let status = unsafe { ENTRY_STATUS.clone() };

        let entry = Entry::new(
            now_seconds,
            settings.core.record_interval_seconds.into(),
            status,
            env_var_list,
        );
        unsafe {
            ENTRY_BUFFER.push(entry);
        }
        let entry_buffer_length = unsafe { ENTRY_BUFFER.len() };

        if entry_buffer_length == ENTRY_BUFFER_MAX_COUNT {
            let storage = Storage::open_as_read_write(
                &database_file_path
                    .as_ref()
                    .expect("Database file path should be valid"),
                settings.core.record_interval_seconds,
            );
            match storage {
                Err(err) => {
                    error!("Could not open storage. {:?}", err);
                    gtk::main_quit();
                    return glib::Continue(false);
                }
                _ => (),
            }
            let mut storage = storage.unwrap();

            unsafe {
                storage.insert_entries(&ENTRY_BUFFER);
                ENTRY_BUFFER.clear();
            }
            let write_result = storage.write_entries();
            match write_result {
                Err(err) => {
                    error!("Could not write to storage. {:#?}", err);
                    gtk::main_quit();
                    return glib::Continue(false);
                }
                _ => (),
            }
            storage.close();
        }

        glib::Continue(true)
    });

    println!("Running Time Tracker Recorder...");
    gtk::main();

    Ok(())
}
