use crate::entries::deduplicate_entries;
use crate::entries::Entry;
use crate::entries::EntryStatus;
use crate::entries::EntryVariablesList;
use crate::entries::RecordRowStatus;
use crate::format_short_executable_name;
use anyhow::{anyhow, Result};
use chrono;
use log::debug;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use sqlite;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn initialize_database(connection: &sqlite::Connection) -> Result<()> {
    // Create database tables to be used for storage.
    //
    // https://www.sqlite.org/foreignkeys.html
    //
    connection.execute(
        "CREATE TABLE records (
              utc_time_seconds INTEGER,
              duration_seconds INTEGER,
              status           INTEGER,
              executable       TEXT,
              var1_name        VARCHAR(255),
              var2_name        VARCHAR(255),
              var3_name        VARCHAR(255),
              var4_name        VARCHAR(255),
              var1_value       TEXT,
              var2_value       TEXT,
              var3_value       TEXT,
              var4_value       TEXT
         );",
    )?;

    Ok(())
}

fn get_last_database_entry(connection: &sqlite::Connection) -> Result<Entry> {
    let mut last_entry = Entry::empty();
    connection.iterate(
        "SELECT utc_time_seconds, duration_seconds, status, executable, var1_name, var2_name, var3_name, var4_name, var1_value, var2_value, var3_value, var4_value
         FROM records
         ORDER BY utc_time_seconds DESC
         LIMIT 1 ;",
        |pairs| {
            for &(column, value) in pairs.iter() {
                debug!("{} = {:?}", column, value);
                match value {
                    Some(v) => match column {
                        "utc_time_seconds" => {
                            last_entry.utc_time_seconds = v.parse::<u64>().unwrap();
                        }
                        "duration_seconds" => {
                            last_entry.duration_seconds = v.parse::<u64>().unwrap();
                        }
                        "status" => {
                            let num = v.parse::<u64>().unwrap();
                            last_entry.status = FromPrimitive::from_u64(num).unwrap();
                        }
                        "executable" => {
                            last_entry.vars.executable = Some(v.to_owned())
                        }
                        "var1_name" => {
                            last_entry.vars.var1_name = Some(v.to_owned())
                        }
                        "var2_name" => {
                            last_entry.vars.var2_name = Some(v.to_owned())
                        }
                        "var3_name" => {
                            last_entry.vars.var3_name = Some(v.to_owned())
                        }
                        "var4_name" => {
                            last_entry.vars.var4_name = Some(v.to_owned())
                        }
                        "var1_value" => {
                            last_entry.vars.var1_value = Some(v.to_owned())
                        }
                        "var2_value" => {
                            last_entry.vars.var2_value = Some(v.to_owned())
                        }
                        "var3_value" => {
                            last_entry.vars.var3_value = Some(v.to_owned())
                        }
                        "var4_value" => {
                            last_entry.vars.var4_value = Some(v.to_owned())
                        }
                        _ => todo!(),
                    },
                    _ => (),
                }
            }
            true // Only one record will be returned anyway.
        },
    )?;
    debug!("Last Entry: {:?}", last_entry);

    Ok(last_entry)
}

fn update_existing_entry_rows_into_database(
    connection: &sqlite::Connection,
    existing_entries_dedup: &Vec<Entry>,
) -> Result<()> {
    let mut cursor = connection
        .prepare(
            "UPDATE records
             SET duration_seconds = :duration_seconds
             WHERE utc_time_seconds = :utc_time_seconds ;",
        )?
        .into_cursor();
    for entry in existing_entries_dedup {
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(
                entry.utc_time_seconds.try_into().unwrap(),
                0,
            )
            .unwrap(),
            chrono::Utc,
        )
        .with_timezone(&chrono::Local);

        let duration = chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
        let duration_formatted = crate::format::format_duration(
            duration,
            crate::format::DurationFormat::HoursMinutesSeconds,
        );
        let time_formatted =
            crate::format::format_datetime(datetime, crate::format::DateTimeFormat::Iso);

        let executable = match &entry.vars.executable {
            Some(value) => {
                let executable_name = format_short_executable_name(&value);
                sqlite::Value::String(executable_name.to_string())
            }
            None => sqlite::Value::Null,
        };

        let var1_name = match &entry.vars.var1_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var2_name = match &entry.vars.var2_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var3_name = match &entry.vars.var3_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var4_name = match &entry.vars.var4_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };

        let var1_value = match &entry.vars.var1_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var2_value = match &entry.vars.var2_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var3_value = match &entry.vars.var3_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var4_value = match &entry.vars.var4_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };

        debug!(
            "UPDATE Entry [ Time: {}, Duration: {}, Status: {:?}, Executable: {:?}, Var1: {:?} = {:?}, Var2: {:?} = {:?}, Var3: {:?} = {:?}, Var4: {:?} = {:?} ]",
            time_formatted,
            duration_formatted,
            entry.status,
            executable,
            var1_name,
            var1_value,
            var2_name,
            var2_value,
            var3_name,
            var3_value,
            var4_name,
            var4_value,
        );

        cursor.bind_by_name(vec![
            (
                ":utc_time_seconds",
                sqlite::Value::Integer(entry.utc_time_seconds as i64),
            ),
            (
                ":duration_seconds",
                sqlite::Value::Integer(entry.duration_seconds as i64),
            ),
        ])?;
        cursor.next()?;
    }

    Ok(())
}

fn insert_new_entry_rows_into_database(
    connection: &sqlite::Connection,
    new_entries_dedup: &Vec<Entry>,
) -> Result<()> {
    let mut cursor = connection
        .prepare(
            "INSERT INTO records (utc_time_seconds,
                                  duration_seconds,
                                  status,
                                  executable,
                                  var1_name,
                                  var2_name,
                                  var3_name,
                                  var4_name,
                                  var1_value,
                                  var2_value,
                                  var3_value,
                                  var4_value)
             VALUES (:utc_time_seconds,
                     :duration_seconds,
                     :status,
                     :executable,
                     :var1_name,
                     :var2_name,
                     :var3_name,
                     :var4_name,
                     :var1_value,
                     :var2_value,
                     :var3_value,
                     :var4_value)",
        )?
        .into_cursor();

    for entry in new_entries_dedup {
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(
                entry.utc_time_seconds.try_into().unwrap(),
                0,
            )
            .unwrap(),
            chrono::Utc,
        )
        .with_timezone(&chrono::Local);

        let duration = chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
        let duration_formatted = crate::format::format_duration(
            duration,
            crate::format::DurationFormat::HoursMinutesSeconds,
        );
        let time_formatted =
            crate::format::format_datetime(datetime, crate::format::DateTimeFormat::Iso);

        let utc_time_seconds = sqlite::Value::Integer(entry.utc_time_seconds as i64);
        let duration_seconds = sqlite::Value::Integer(entry.duration_seconds as i64);

        let status_num = match entry.status.to_i64() {
            Some(value) => value,
            None => panic!("Invalid EntryStatus."),
        };
        let status = sqlite::Value::Integer(status_num);

        let executable = match &entry.vars.executable {
            Some(value) => {
                let executable_name = format_short_executable_name(&value);
                sqlite::Value::String(executable_name.to_string())
            }
            None => sqlite::Value::Null,
        };

        let var1_name = match &entry.vars.var1_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var2_name = match &entry.vars.var2_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var3_name = match &entry.vars.var3_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var4_name = match &entry.vars.var4_name {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };

        let var1_value = match &entry.vars.var1_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var2_value = match &entry.vars.var2_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var3_value = match &entry.vars.var3_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };
        let var4_value = match &entry.vars.var4_value {
            Some(value) => sqlite::Value::String(value.to_string()),
            None => sqlite::Value::Null,
        };

        debug!("INSERT Entry [ Time: {}, Duration: {}, Status: {:?}, Executable: {:?}, Var1: {:?} = {:?}, Var2: {:?} = {:?}, Var3: {:?} = {:?}, Var4: {:?} = {:?} ]",
               time_formatted,
               duration_formatted,
               entry.status,
               &executable,
               var1_name,
               var1_value,
               var2_name,
               var2_value,
               var3_name,
               var3_value,
               var4_name,
               var4_value,
        );

        cursor.bind_by_name(vec![
            (":utc_time_seconds", utc_time_seconds),
            (":duration_seconds", duration_seconds),
            (":status", status),
            (":executable", executable),
            (":var1_name", var1_name),
            (":var2_name", var2_name),
            (":var3_name", var3_name),
            (":var4_name", var4_name),
            (":var1_value", var1_value),
            (":var2_value", var2_value),
            (":var3_value", var3_value),
            (":var4_value", var4_value),
        ])?;
        cursor.next()?;
    }

    Ok(())
}

pub struct Storage {
    connection: sqlite::Connection,
    entries: Vec<Entry>,
    record_interval_seconds: u64,
}

impl Storage {
    fn open(
        database_file_path: &Path,
        record_interval_seconds: u64,
        auto_create_database_file: bool,
    ) -> Result<Storage> {
        debug!("Opened Time Tracker Storage.");

        debug!("Storage file: {:?}", database_file_path);
        let file_exists = database_file_path.is_file();

        if auto_create_database_file == false && file_exists == false {
            return Err(anyhow!(
                "Database storage file does not exist: {}",
                database_file_path.display()
            ));
        }

        let db_open_flags = sqlite::OpenFlags::new()
            .set_create()
            .set_read_write()
            .set_full_mutex();
        let connection = sqlite::Connection::open_with_flags(&database_file_path, db_open_flags)?;

        if !file_exists {
            initialize_database(&connection)?;

            // Change the permissions on the database file, so
            // that ONLY the current user can read it. This
            // reduces the issue of privacy.
            let f =
                File::open(database_file_path).expect("Could not open file to set permissions.");
            let mut permissions = f
                .metadata()
                .expect("Could not get database file metadata.")
                .permissions();
            permissions.set_mode(0o600);
            f.set_permissions(permissions)
                .expect("Could not open file to set permissions.");
        }

        let entries = Vec::<_>::new();
        Ok(Storage {
            connection,
            entries,
            record_interval_seconds,
        })
    }

    pub fn open_as_read_only(
        database_file_path: &Path,
        record_interval_seconds: u64,
    ) -> Result<Storage> {
        let auto_create_database_file = false;
        Storage::open(
            database_file_path,
            record_interval_seconds,
            auto_create_database_file,
        )
    }

    pub fn open_as_read_write(
        database_file_path: &Path,
        record_interval_seconds: u64,
    ) -> Result<Storage> {
        let auto_create_database_file = true;
        Storage::open(
            database_file_path,
            record_interval_seconds,
            auto_create_database_file,
        )
    }

    pub fn insert_entries(&mut self, entries: &Vec<Entry>) {
        for entry in entries {
            debug!("Insert Entry: {:?}", entry);
            self.entries.push(entry.clone());
        }
    }

    // TODO: Fix how we deal with entries that wrap over the start and
    // end time arguments. For example, if an entry spans from Monday
    // 11:50pm to Tuesday 0:10am, this entry may be skipped or
    // included. What we want is to cut off such an entry and "clamp"
    // the time values of the entries to be only with-in the start/end
    // time parameters.
    pub fn read_entries(
        &mut self,
        start_utc_time_seconds: u64,
        end_utc_time_seconds: u64,
    ) -> Result<Vec<Entry>> {
        let mut cursor = self
            .connection
            .prepare(
                "SELECT *
                 FROM records
                 WHERE utc_time_seconds > :start_utc_time_seconds
                       AND utc_time_seconds < :end_utc_time_seconds
                 ORDER BY utc_time_seconds ASC ;",
            )?
            .into_cursor();
        cursor.bind_by_name(vec![
            (
                ":start_utc_time_seconds",
                sqlite::Value::Integer(start_utc_time_seconds as i64),
            ),
            (
                ":end_utc_time_seconds",
                sqlite::Value::Integer(end_utc_time_seconds as i64),
            ),
        ])?;

        let mut entries = Vec::<Entry>::new();
        while let Some(row) = cursor.next()? {
            debug!("row = {:?}", row);
            let utc_time_seconds = row[0].as_integer().unwrap();
            let duration_seconds = row[1].as_integer().unwrap();
            let status_num = row[2].as_integer().unwrap();
            let status: EntryStatus = FromPrimitive::from_u64(status_num.try_into()?).unwrap();

            let executable: Option<String> = match &row[4] {
                sqlite::Value::String(value) => Some(value.clone()),
                sqlite::Value::Null => None,
                _ => panic!("executable should only be an String or Null type."),
            };

            let mut vars = EntryVariablesList::empty();
            vars.executable = executable; // Some("bash".to_string());

            let entry = Entry::new(
                utc_time_seconds.try_into()?,
                duration_seconds.try_into()?,
                status,
                vars,
            );
            entries.push(entry);
        }
        Ok(entries)
    }

    pub fn write_entries(&mut self) -> Result<()> {
        // TODO: When we write entries into the database the first time
        // we should first find the variable names in the database,
        // and add them if they are missing, then cache the 'variable
        // name ids' to avoid future database calls for the entire
        // lifetime of the program.

        // TODO: In addition to caching the "variable name ids", we
        // may also want to keep an in-memory least-recently-used
        // cache of "variable value ids", but with a fixed amount of
        // enties allowed - this is a "speed vs size" trade-off, to
        // avoid database queries calls.

        // Execute the entires and close the SQLite database
        // connection.
        self.connection.execute("BEGIN TRANSACTION;")?;

        let last_entry = get_last_database_entry(&self.connection)?;

        let mut entries_dedup = Vec::<Entry>::new();
        let mut entry_row_statuses = Vec::<RecordRowStatus>::new();

        deduplicate_entries(
            &last_entry,
            &self.entries,
            self.record_interval_seconds,
            &mut entries_dedup,
            &mut entry_row_statuses,
        );

        let new_entries_dedup: Vec<Entry> = entries_dedup
            .iter()
            .zip(&entry_row_statuses)
            .filter(|x| x.1 == &RecordRowStatus::New)
            .map(|x| x.0.clone())
            .collect();
        let existing_entries_dedup: Vec<Entry> = entries_dedup
            .iter()
            .zip(&entry_row_statuses)
            .filter(|x| x.1 == &RecordRowStatus::Existing)
            .map(|x| x.0.clone())
            .collect();

        update_existing_entry_rows_into_database(&self.connection, &existing_entries_dedup)?;
        insert_new_entry_rows_into_database(&self.connection, &new_entries_dedup)?;

        self.connection.execute("END TRANSACTION;")?;

        Ok(())
    }

    pub fn close(&mut self) {
        // close the SQLite database connection.
        debug!("Closed Time Tracker Storage (currently does nothing).");
    }
}
