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
use rusqlite;
use rusqlite::named_params;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

// The indexes of the fields in the database, used to index into
// queried rows.
const INDEX_UTC_TIME_SECONDS: usize = 0;
const INDEX_DURATION_SECONDS: usize = 1;
const INDEX_STATUS: usize = 2;
const INDEX_EXECUTABLE: usize = 3;
const INDEX_VAR1_NAME: usize = 4;
const INDEX_VAR2_NAME: usize = 5;
const INDEX_VAR3_NAME: usize = 6;
const INDEX_VAR4_NAME: usize = 7;
const INDEX_VAR5_NAME: usize = 8;
const INDEX_VAR1_VALUE: usize = 9;
const INDEX_VAR2_VALUE: usize = 10;
const INDEX_VAR3_VALUE: usize = 11;
const INDEX_VAR4_VALUE: usize = 12;
const INDEX_VAR5_VALUE: usize = 13;

/// The maximum number of environment variables that can be stored in
/// the database.
pub const ENVIRONMENT_VARIABLE_NAMES_MAX_COUNT: usize = 5;

fn initialize_database(connection: &rusqlite::Connection) -> Result<()> {
    debug!("Initialize Database...");

    // Create database tables to be used for storage.
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
              var5_name        VARCHAR(255),
              var1_value       TEXT,
              var2_value       TEXT,
              var3_value       TEXT,
              var4_value       TEXT,
              var5_value       TEXT
         );",
        (), // no parameters needed to create a table.
    )?;

    Ok(())
}

fn get_last_database_entry(connection: &rusqlite::Connection) -> Result<Entry> {
    let mut statement = connection.prepare(
        "SELECT utc_time_seconds, duration_seconds, status, executable, var1_name, var2_name, var3_name, var4_name, var5_name, var1_value, var2_value, var3_value, var4_value, var5_value
         FROM records
         ORDER BY utc_time_seconds DESC
         LIMIT 1 ;"
    )?;

    let mut last_entry = Entry::empty();
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        last_entry.utc_time_seconds = row.get_unwrap::<usize, u64>(INDEX_UTC_TIME_SECONDS);
        last_entry.duration_seconds = row.get_unwrap::<usize, u64>(INDEX_DURATION_SECONDS);
        let status_num = row.get_unwrap::<usize, i64>(INDEX_STATUS);
        last_entry.status = FromPrimitive::from_i64(status_num).unwrap();
        last_entry.vars.executable = row.get_unwrap::<usize, Option<String>>(INDEX_EXECUTABLE);
        last_entry.vars.var1_name = row.get_unwrap::<usize, Option<String>>(INDEX_VAR1_NAME);
        last_entry.vars.var2_name = row.get_unwrap::<usize, Option<String>>(INDEX_VAR2_NAME);
        last_entry.vars.var3_name = row.get_unwrap::<usize, Option<String>>(INDEX_VAR3_NAME);
        last_entry.vars.var4_name = row.get_unwrap::<usize, Option<String>>(INDEX_VAR4_NAME);
        last_entry.vars.var5_name = row.get_unwrap::<usize, Option<String>>(INDEX_VAR5_NAME);
        last_entry.vars.var1_value = row.get_unwrap::<usize, Option<String>>(INDEX_VAR1_VALUE);
        last_entry.vars.var2_value = row.get_unwrap::<usize, Option<String>>(INDEX_VAR2_VALUE);
        last_entry.vars.var3_value = row.get_unwrap::<usize, Option<String>>(INDEX_VAR3_VALUE);
        last_entry.vars.var4_value = row.get_unwrap::<usize, Option<String>>(INDEX_VAR4_VALUE);
        last_entry.vars.var5_value = row.get_unwrap::<usize, Option<String>>(INDEX_VAR5_VALUE);
    }
    debug!("Last Entry: {:?}", last_entry);

    Ok(last_entry)
}

fn utc_seconds_to_datetime_local(utc_time_seconds: u64) -> chrono::DateTime<chrono::Local> {
    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        chrono::NaiveDateTime::from_timestamp_opt(utc_time_seconds.try_into().unwrap(), 0).unwrap(),
        chrono::Utc,
    )
    .with_timezone(&chrono::Local)
}

fn update_existing_entry_rows_into_database(
    connection: &rusqlite::Connection,
    existing_entries_dedup: &Vec<Entry>,
) -> Result<()> {
    let mut statement = connection.prepare(
        "UPDATE records
             SET duration_seconds = :duration_seconds
             WHERE utc_time_seconds = :utc_time_seconds ;",
    )?;
    for entry in existing_entries_dedup {
        let datetime = utc_seconds_to_datetime_local(entry.utc_time_seconds);

        let duration = chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
        let duration_formatted = crate::format::format_duration(
            duration,
            crate::format::DurationFormat::HoursMinutesSeconds,
        );
        let time_formatted =
            crate::format::format_datetime(datetime, crate::format::DateTimeFormat::Iso);

        let executable = match &entry.vars.executable {
            Some(value) => {
                let executable_name = format_short_executable_name(value);
                rusqlite::types::Value::Text(executable_name.to_string())
            }
            None => rusqlite::types::Value::Null,
        };

        let var1_name = convert_entry_var_to_sql_string_value(&entry.vars.var1_name);
        let var2_name = convert_entry_var_to_sql_string_value(&entry.vars.var2_name);
        let var3_name = convert_entry_var_to_sql_string_value(&entry.vars.var3_name);
        let var4_name = convert_entry_var_to_sql_string_value(&entry.vars.var4_name);
        let var5_name = convert_entry_var_to_sql_string_value(&entry.vars.var5_name);

        let var1_value = convert_entry_var_to_sql_string_value(&entry.vars.var1_value);
        let var2_value = convert_entry_var_to_sql_string_value(&entry.vars.var2_value);
        let var3_value = convert_entry_var_to_sql_string_value(&entry.vars.var3_value);
        let var4_value = convert_entry_var_to_sql_string_value(&entry.vars.var4_value);
        let var5_value = convert_entry_var_to_sql_string_value(&entry.vars.var5_value);

        debug!(
            "UPDATE Entry [ Time: {}, Duration: {}, Status: {:?}, Executable: {:?}, Var1: {:?} = {:?}, Var2: {:?} = {:?}, Var3: {:?} = {:?}, Var4: {:?} = {:?}, Var5: {:?} = {:?} ]",
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
            var5_name,
            var5_value,
        );

        statement.execute(named_params! {
            ":utc_time_seconds": rusqlite::types::Value::Integer(entry.utc_time_seconds as i64),
            ":duration_seconds": rusqlite::types::Value::Integer(entry.duration_seconds as i64)
        })?;
    }

    Ok(())
}

fn convert_entry_var_to_sql_string_value(
    entry_var_name: &Option<String>,
) -> rusqlite::types::Value {
    match &entry_var_name {
        Some(value) => rusqlite::types::Value::Text(value.to_string()),
        None => rusqlite::types::Value::Null,
    }
}

fn convert_sql_value_to_option_string(sql_value: &rusqlite::types::Value) -> Option<String> {
    match sql_value {
        rusqlite::types::Value::Text(value) => Some(value.clone()),
        rusqlite::types::Value::Null => None,
        _ => panic!("SQLite value can only be an String or Null type."),
    }
}

fn insert_new_entry_rows_into_database(
    connection: &rusqlite::Connection,
    new_entries_dedup: &Vec<Entry>,
) -> Result<()> {
    let mut statement = connection.prepare(
        "INSERT INTO records (utc_time_seconds,
                                  duration_seconds,
                                  status,
                                  executable,
                                  var1_name,
                                  var2_name,
                                  var3_name,
                                  var4_name,
                                  var5_name,
                                  var1_value,
                                  var2_value,
                                  var3_value,
                                  var4_value,
                                  var5_value)
             VALUES (:utc_time_seconds,
                     :duration_seconds,
                     :status,
                     :executable,
                     :var1_name,
                     :var2_name,
                     :var3_name,
                     :var4_name,
                     :var5_name,
                     :var1_value,
                     :var2_value,
                     :var3_value,
                     :var4_value,
                     :var5_value)",
    )?;

    for entry in new_entries_dedup {
        let datetime = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
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

        let utc_time_seconds = rusqlite::types::Value::Integer(entry.utc_time_seconds as i64);
        let duration_seconds = rusqlite::types::Value::Integer(entry.duration_seconds as i64);

        let status_num = match entry.status.to_i64() {
            Some(value) => value,
            None => panic!("Invalid EntryStatus."),
        };
        let status = rusqlite::types::Value::Integer(status_num);

        let executable = match &entry.vars.executable {
            Some(value) => {
                let executable_name = format_short_executable_name(value);
                rusqlite::types::Value::Text(executable_name.to_string())
            }
            None => rusqlite::types::Value::Null,
        };

        let var1_name = convert_entry_var_to_sql_string_value(&entry.vars.var1_name);
        let var2_name = convert_entry_var_to_sql_string_value(&entry.vars.var2_name);
        let var3_name = convert_entry_var_to_sql_string_value(&entry.vars.var3_name);
        let var4_name = convert_entry_var_to_sql_string_value(&entry.vars.var4_name);
        let var5_name = convert_entry_var_to_sql_string_value(&entry.vars.var5_name);

        let var1_value = convert_entry_var_to_sql_string_value(&entry.vars.var1_value);
        let var2_value = convert_entry_var_to_sql_string_value(&entry.vars.var2_value);
        let var3_value = convert_entry_var_to_sql_string_value(&entry.vars.var3_value);
        let var4_value = convert_entry_var_to_sql_string_value(&entry.vars.var4_value);
        let var5_value = convert_entry_var_to_sql_string_value(&entry.vars.var5_value);

        debug!("INSERT Entry [ Time: {}, Duration: {}, Status: {:?}, Executable: {:?}, Var1: {:?} = {:?}, Var2: {:?} = {:?}, Var3: {:?} = {:?}, Var4: {:?} = {:?}, Var5: {:?} = {:?} ]",
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
               var5_name,
               var5_value,
        );

        statement.execute(named_params! {
            ":utc_time_seconds": utc_time_seconds,
            ":duration_seconds": duration_seconds,
            ":status": status,
            ":executable": executable,
            ":var1_name": var1_name,
            ":var2_name": var2_name,
            ":var3_name": var3_name,
            ":var4_name": var4_name,
            ":var5_name": var5_name,
            ":var1_value": var1_value,
            ":var2_value": var2_value,
            ":var3_value": var3_value,
            ":var4_value": var4_value,
            ":var5_value": var5_value,
        })?;
    }

    Ok(())
}

// Store read-only entries.
//
// Allows filtering the full list of entries by a sub-set of
// times/dates (without having to fetch data from the database).
#[derive(Debug)]
pub struct Entries {
    start_datetime: chrono::DateTime<chrono::Local>,
    end_datetime: chrono::DateTime<chrono::Local>,
    entries: Vec<Entry>,
}

impl Entries {
    pub fn builder() -> EntriesBuilder {
        EntriesBuilder::default()
    }

    pub fn start_datetime(&self) -> chrono::DateTime<chrono::Local> {
        self.start_datetime
    }

    pub fn end_datetime(&self) -> chrono::DateTime<chrono::Local> {
        self.end_datetime
    }

    // Get a slice of all the entries.
    pub fn all_entries(&self) -> &[Entry] {
        &self.entries[..]
    }

    // Get a slice of the entries for the datetime range given.
    pub fn datetime_range_entries(
        &self,
        start_datetime: chrono::DateTime<chrono::Local>,
        end_datetime: chrono::DateTime<chrono::Local>,
    ) -> &[Entry] {
        let start_of_time = start_datetime.timestamp() as u64;
        let end_of_time = end_datetime.timestamp() as u64;

        let mut start_index = 0;
        let mut end_index = 0;
        for (i, entry) in self.entries.iter().enumerate() {
            if (entry.utc_time_seconds > start_of_time) && (entry.utc_time_seconds < end_of_time) {
                start_index = std::cmp::min(start_index, i);
                end_index = std::cmp::max(end_index, i);
            }
        }

        &self.entries[start_index..end_index]
    }

    pub fn is_datetime_range_empty(
        &self,
        start_datetime: chrono::DateTime<chrono::Local>,
        end_datetime: chrono::DateTime<chrono::Local>,
    ) -> bool {
        self.datetime_range_entries(start_datetime, end_datetime)
            .is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Default)]
pub struct EntriesBuilder {
    start_datetime: chrono::DateTime<chrono::Local>,
    end_datetime: chrono::DateTime<chrono::Local>,
    entries: Vec<Entry>,
}

impl EntriesBuilder {
    pub fn new() -> EntriesBuilder {
        EntriesBuilder {
            start_datetime: chrono::DateTime::<chrono::Local>::MIN_UTC.into(),
            end_datetime: chrono::DateTime::<chrono::Local>::MAX_UTC.into(),
            entries: Vec::new(),
        }
    }

    pub fn start_datetime(mut self, value: chrono::DateTime<chrono::Local>) -> EntriesBuilder {
        self.start_datetime = value;
        self
    }

    pub fn end_datetime(mut self, value: chrono::DateTime<chrono::Local>) -> EntriesBuilder {
        self.end_datetime = value;
        self
    }

    pub fn entries(mut self, entries: Vec<Entry>) -> EntriesBuilder {
        self.entries = entries;
        self
    }

    pub fn build(self) -> Entries {
        Entries {
            start_datetime: self.start_datetime,
            end_datetime: self.end_datetime,
            entries: self.entries,
        }
    }
}

pub struct Storage {
    connection: rusqlite::Connection,
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

        if !auto_create_database_file && !file_exists {
            return Err(anyhow!(
                "Database storage file does not exist: {}",
                database_file_path.display()
            ));
        }

        let db_open_flags = rusqlite::OpenFlags::SQLITE_OPEN_CREATE
            | rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let connection = rusqlite::Connection::open_with_flags(database_file_path, db_open_flags)?;

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
    ) -> Result<Entries> {
        let mut statement = self.connection.prepare(
            "SELECT utc_time_seconds, duration_seconds, status,
                        executable,
                        var1_name, var2_name, var3_name, var4_name, var5_name,
                        var1_value, var2_value, var3_value, var4_value, var5_value
                 FROM records
                 WHERE utc_time_seconds > :start_utc_time_seconds
                       AND utc_time_seconds < :end_utc_time_seconds
                 ORDER BY utc_time_seconds ASC ;",
        )?;
        let mut rows = statement.query(named_params! {
            ":start_utc_time_seconds": rusqlite::types::Value::Integer(start_utc_time_seconds as i64),
            ":end_utc_time_seconds": rusqlite::types::Value::Integer(end_utc_time_seconds as i64),
        })?;

        let mut entries = Vec::<Entry>::new();
        while let Some(row) = rows.next()? {
            let utc_time_seconds: u64 = row.get_unwrap(INDEX_UTC_TIME_SECONDS);
            let duration_seconds: u64 = row.get_unwrap(INDEX_DURATION_SECONDS);
            let status_num: u64 = row.get_unwrap(INDEX_STATUS);
            let status: EntryStatus = FromPrimitive::from_u64(status_num).unwrap();

            let mut vars = EntryVariablesList::empty();
            vars.executable = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_EXECUTABLE));
            vars.var1_name = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR1_NAME));
            vars.var2_name = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR2_NAME));
            vars.var3_name = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR3_NAME));
            vars.var4_name = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR4_NAME));
            vars.var5_name = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR5_NAME));
            vars.var1_value = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR1_VALUE));
            vars.var2_value = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR2_VALUE));
            vars.var3_value = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR3_VALUE));
            vars.var4_value = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR4_VALUE));
            vars.var5_value = convert_sql_value_to_option_string(&row.get_unwrap(INDEX_VAR5_VALUE));

            let entry = Entry::new(utc_time_seconds, duration_seconds, status, vars);
            entries.push(entry);
        }

        Ok(Entries::builder()
            .start_datetime(utc_seconds_to_datetime_local(start_utc_time_seconds))
            .end_datetime(utc_seconds_to_datetime_local(end_utc_time_seconds))
            .entries(entries)
            .build())
    }

    pub fn write_entries(&mut self) -> Result<()> {
        // Execute the entires and close the SQLite database
        // connection.
        self.connection.execute("BEGIN TRANSACTION;", ())?;

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

        self.connection.execute("END TRANSACTION;", ())?;

        Ok(())
    }

    pub fn close(&mut self) {
        // close the SQLite database connection.
        debug!("Closed Time Tracker Storage.");
    }
}
