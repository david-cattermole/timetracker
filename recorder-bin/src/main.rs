use crate::linux_process::find_process_ids_by_user_and_executable_name;
use crate::linux_process::get_process_id_executable_name;
use crate::linux_process::get_user_id_running_process_id;
use crate::linux_process::read_process_environment_variables;
use crate::linux_process::terminate_processes;
use crate::linux_signal::install_signal_handler;
use crate::settings::CommandArguments;
use crate::settings::CommandModes;
use crate::settings::RecorderAppSettings;
use anyhow::{bail, Result};
use clap::Parser;
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use std::path::Path;
use std::path::PathBuf;
use std::sync;
use std::sync::Mutex;
use std::thread;
use std::time;
use timetracker_core::entries::Entry;
use timetracker_core::entries::EntryStatus;
use timetracker_core::entries::EntryVariablesList;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::settings::RECORD_INTERVAL_SECONDS;
use timetracker_core::settings::USER_IS_IDLE_LIMIT_SECONDS;
use timetracker_core::storage::Storage;

#[cfg(target_os = "linux")]
mod linux_process;
#[cfg(target_os = "linux")]
mod linux_signal;
#[cfg(target_os = "linux")]
mod linux_x11;

mod settings;

/// How many enties are stored in memory before being saved to the
/// storage.
const ENTRY_BUFFER_MAX_COUNT: usize = 10;

/// The global buffer of entries stored in memory, waiting to be
/// written to storage.
static mut ENTRY_BUFFER: Lazy<Mutex<Vec<Entry>>> = Lazy::new(|| Mutex::new(vec![]));

/// The global status of the user; Is the user active or idle?
static mut ENTRY_STATUS: EntryStatus = EntryStatus::Uninitialized;

/// The database file path is stored so the signal handler clean up
/// function (named "handle_signal") can use it to write data to to
/// the database when exiting the process.
static mut CLEANUP_DATABASE_FILE_PATH: Lazy<Mutex<PathBuf>> =
    Lazy::new(|| Mutex::new(PathBuf::new()));

/// The name of this executable file name.
const THIS_EXECUTABLE_NAME: &str = "timetracker-recorder";

/// Writes data to the database, and retries multiple times until
/// success can be made, or a timer runs out.
fn write_data_to_storage(database_file_path: &Path) -> Result<()> {
    let now = time::SystemTime::now();

    let mut wait_duration = time::Duration::from_millis(1);
    // 8 seconds is chosen to stop the storage attempts before the
    // next round of storage read/write attempts are made.
    let total_allowed_wait_seconds =
        ((RECORD_INTERVAL_SECONDS as f32 * ENTRY_BUFFER_MAX_COUNT as f32) * 0.8) as u64;
    let total_allowed_wait_duration = time::Duration::from_secs(total_allowed_wait_seconds);
    let total_allowed_attempts = 10;
    for attempt_number in 0..=(total_allowed_attempts + 1) {
        if attempt_number > 0 {
            error!("Attempt #{}.", attempt_number);

            let mut do_exit = false;
            if attempt_number >= total_allowed_attempts {
                error!("All {} attempts failed. Exiting.", attempt_number);
                do_exit = true;
            }
            let has_waited = now.elapsed()?;
            if has_waited > total_allowed_wait_duration {
                error!(
                    "Running {} attempts has taken longer than {:?}. Exiting...",
                    attempt_number, total_allowed_wait_duration
                );
                do_exit = true;
            }
            if do_exit {
                // This will stop the full program, along with all
                // threads (including the main thread).
                std::process::abort();
            }

            thread::sleep(wait_duration);
            wait_duration += wait_duration * 2;
        }

        let storage = Storage::open_as_read_write(database_file_path, RECORD_INTERVAL_SECONDS);
        if let Err(err) = storage {
            error!("Could not open storage. {:?}", err);
            continue;
        }
        let mut storage = storage?;

        unsafe {
            let mut data = ENTRY_BUFFER.lock().unwrap();
            storage.insert_entries(&data);
            let _ = &data.clear();
        }
        let write_result = storage.write_entries();
        if let Err(err) = write_result {
            error!("Could not write to storage. {:#?}", err);
            continue;
        }
        storage.close();

        if attempt_number == 0 {
            debug!("Successfully written to storage.");
        } else {
            warn!(
                "Successfully written to storage with {} retries.",
                attempt_number
            );
        }
        break;
    }

    Ok(())
}

/// Function that gets called when this process is given a signal
/// (such as 'SIGINT' number 2 or 'SIGTERM' number 15) and told to
/// terminate.
extern "C" fn handle_signal(signal_number: libc::c_int) {
    warn!("Received signal {}, exiting gracefully...", signal_number);

    let database_file_path = unsafe { &CLEANUP_DATABASE_FILE_PATH.lock().unwrap() };
    write_data_to_storage(database_file_path).unwrap();

    // This will stop the full program, along with all threads
    // (including the main thread).
    std::process::abort();
}

/// Run to start recording activity.
fn start_recording(
    _args: &CommandArguments,
    settings: RecorderAppSettings,
    terminate_existing_processes: bool,
) -> Result<()> {
    println!("Starting Time Tracker Recorder...");

    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    )
    .expect("Database file path should be valid");
    println!("Database file: {:?}", database_file_path);

    // Store a copy of the database file path in static memory, so the
    // "handle_signal" function can use it.
    unsafe {
        let mut cleanup_database_file_path = CLEANUP_DATABASE_FILE_PATH.lock().unwrap();
        *cleanup_database_file_path = database_file_path.clone();
    };

    // Signal handlers allow us to clean up and write data to the
    // database before the process shuts down.
    install_signal_handler(libc::SIGINT, handle_signal as usize);
    install_signal_handler(libc::SIGTERM, handle_signal as usize);

    let this_process_id = std::process::id();
    let this_user_id = get_user_id_running_process_id(this_process_id)?;
    let running_process_ids = find_process_ids_by_user_and_executable_name(
        THIS_EXECUTABLE_NAME,
        this_user_id,
        this_process_id,
    )?;
    if !running_process_ids.is_empty() {
        if terminate_existing_processes {
            terminate_processes(&running_process_ids)?;
        } else {
            error!(
                "{} is already running, found running process ids {:?}.",
                THIS_EXECUTABLE_NAME, running_process_ids
            );
            error!("Rerun with --terminate-existing-processes flag to kill the running processes.");
            return Ok(());
        }
    }

    // TODO: When this this function is meant to go into a loop and
    // query X11, instead we should make a child process that queries
    // the X11 stuff, because it can be a little unstable in weird
    // edge cases (that can happen on KDE). Therefore we should start
    // a new child process that does the real work, and this
    // ("parent") process will wait for the child-process to exit then
    // re-run the same command when it errors. This will mean that no
    // matter what happens the recorder will always be restarted if a
    // panic happens.

    gtk::init()?;

    let (tx, rx) = sync::mpsc::channel();

    // A second thread is used to avoid a congested/slow storage
    // read/write from slowing down or messing up the recording of
    // user activity, and causing instability or a panic.
    thread::spawn(move || loop {
        rx.recv()
            .expect("Should have recieved a value from the main thread.");
        write_data_to_storage(&database_file_path).unwrap();
    });

    let record_interval_seconds = RECORD_INTERVAL_SECONDS;
    let user_is_idle_limit_seconds = USER_IS_IDLE_LIMIT_SECONDS;
    let interval_seconds = record_interval_seconds.try_into()?;
    let _source_id = glib::source::timeout_add_seconds_local(interval_seconds, move || {
        let idle_time_sec = linux_x11::get_user_idle_time_from_x11();
        if idle_time_sec > user_is_idle_limit_seconds {
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

        let process_id = linux_x11::get_active_window_process_id_from_x11().unwrap();
        debug!("Process ID: {:?}", process_id);
        match process_id {
            0 => (),
            _ => {
                let environ_vars = read_process_environment_variables(process_id);
                match environ_vars {
                    Ok(env_vars) => {
                        env_var_list.replace_with_environ_vars(&env_vars);
                        let exec_name = get_process_id_executable_name(process_id);
                        match exec_name {
                            Ok(exec_name) => env_var_list.executable = Some(exec_name),
                            Err(err) => {
                                warn!(
                                    "Could not get process id executable name: pid={:?} err={:?}",
                                    process_id, err
                                );
                                env_var_list.executable = None;
                            }
                        }
                    }
                    Err(err) => warn!(
                        "Could not read process environment variables: pid={:?} err={:?}",
                        process_id, err
                    ),
                }
            }
        };

        let now_seconds = chrono::Utc::now().timestamp() as u64;
        debug!("Time: {:?}", now_seconds);

        let status = unsafe { ENTRY_STATUS };

        let entry = Entry::new(now_seconds, record_interval_seconds, status, env_var_list);

        let entry_buffer_length = unsafe {
            let mut data = ENTRY_BUFFER.lock().unwrap();
            let _ = &data.push(entry);
            data.len()
        };

        if entry_buffer_length == ENTRY_BUFFER_MAX_COUNT {
            tx.send(true).unwrap();
        }

        glib::Continue(true)
    });

    println!("Running Time Tracker Recorder...");
    gtk::main();

    Ok(())
}

/// Stops recording activity by finding existing processes and sending
/// a SIGTERM signal.
fn stop_recording() -> Result<()> {
    println!("Stopping Time Tracker Recorder...");

    let this_process_id = std::process::id();
    let this_user_id = get_user_id_running_process_id(this_process_id)?;
    let running_process_ids = find_process_ids_by_user_and_executable_name(
        THIS_EXECUTABLE_NAME,
        this_user_id,
        this_process_id,
    )?;
    info!(
        "Found {} running process ids for {}: {:?}.",
        running_process_ids.len(),
        THIS_EXECUTABLE_NAME,
        running_process_ids
    );

    if running_process_ids.is_empty() {
        warn!("No {} processes found to stop.", THIS_EXECUTABLE_NAME);
    } else {
        terminate_processes(&running_process_ids)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("TIMETRACKER_LOG", "warn")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = RecorderAppSettings::new(&args);
    if settings.is_err() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings?;
    debug!("Settings validated: {:#?}", settings);

    match &args.command {
        CommandModes::Start {
            terminate_existing_processes,
        } => start_recording(&args, settings, *terminate_existing_processes)?,
        CommandModes::Stop => stop_recording()?,
    }

    Ok(())
}
