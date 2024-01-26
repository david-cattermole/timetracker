use crate::settings::CommandArguments;
use crate::settings::DumpAppSettings;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use log::debug;
use std::io::prelude::*;
use std::time::SystemTime;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::settings::RECORD_INTERVAL_SECONDS;
use timetracker_core::storage::Entries;
use timetracker_core::storage::Storage;
use timetracker_print_lib::print::get_relative_week_start_end;

mod settings;

// CSV Spec: Each record is located on a separate line,
// delimited by a line break (CRLF).
static LINE_END: &[u8] = "\r\n".as_bytes();

// The CSV File Format header is described here:
// https://www.rfc-editor.org/rfc/rfc4180#section-2
static HEADER_LINE: &[u8] = concat!(
    "utc_time_seconds,duration_seconds,",
    "status,executable,",
    "var1_name,var1_value,",
    "var2_name,var2_value,",
    "var3_name,var3_value,",
    "var4_name,var4_value,",
    "var5_name,var5_value",
)
.as_bytes();

fn convert_to_csv_string_value(entry_var_name: &Option<String>) -> String {
    match &entry_var_name {
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}

fn generate_csv_formated_lines(entries: &Entries, lines: &mut Vec<String>) -> Result<()> {
    for entry in entries.all_entries() {
        let line = format!(
            concat!(
                "{utc_time_seconds},{duration_seconds},",
                "{status:?},{executable},",
                "{var1_name},{var1_value},",
                "{var2_name},{var2_value},",
                "{var3_name},{var3_value},",
                "{var4_name},{var4_value},",
                "{var5_name},{var5_value}"
            ),
            utc_time_seconds = entry.utc_time_seconds,
            duration_seconds = entry.duration_seconds,
            status = entry.status,
            executable = convert_to_csv_string_value(&entry.vars.executable),
            var1_name = convert_to_csv_string_value(&entry.vars.var1_name),
            var1_value = convert_to_csv_string_value(&entry.vars.var1_value),
            var2_name = convert_to_csv_string_value(&entry.vars.var2_name),
            var2_value = convert_to_csv_string_value(&entry.vars.var2_value),
            var3_name = convert_to_csv_string_value(&entry.vars.var3_name),
            var3_value = convert_to_csv_string_value(&entry.vars.var3_value),
            var4_name = convert_to_csv_string_value(&entry.vars.var4_name),
            var4_value = convert_to_csv_string_value(&entry.vars.var4_value),
            var5_name = convert_to_csv_string_value(&entry.vars.var5_name),
            var5_value = convert_to_csv_string_value(&entry.vars.var5_value),
        );
        lines.push(line);
    }
    Ok(())
}

fn dump_database(
    args: &CommandArguments,
    settings: &DumpAppSettings,
    output_lines: &mut Vec<String>,
) -> Result<()> {
    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    );

    let mut storage = Storage::open_as_read_only(
        &database_file_path.expect("Database file path should be valid"),
        RECORD_INTERVAL_SECONDS,
    )?;

    let relative_week = if args.last_week {
        -1
    } else {
        args.relative_week
    };

    // 'relative_week' is added to the week number to find. A value of
    // '-1' will get the previous week, a value of '0' will get the
    // current week, and a value of '1' will get the next week (which
    // shouldn't really give any results, so it's probably pointless).
    let week_datetime_pair = get_relative_week_start_end(relative_week)?;

    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let week_start_of_time = week_start_datetime.timestamp() as u64;
    let week_end_of_time = week_end_datetime.timestamp() as u64;
    let week_entries = storage.read_entries(week_start_of_time, week_end_of_time)?;

    generate_csv_formated_lines(&week_entries, output_lines)
}

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("TIMETRACKER_LOG", "warn")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = DumpAppSettings::new(&args);
    if settings.is_err() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings?;
    debug!("Settings validated: {:#?}", settings);

    let now = SystemTime::now();

    let mut lines = Vec::new();
    dump_database(&args, &settings, &mut lines)?;

    if !lines.is_empty() {
        match args.output_file {
            Some(file_path) => {
                let f = std::fs::File::create(file_path)?;
                let mut writer = std::io::BufWriter::new(f);
                writer.write(HEADER_LINE)?;
                writer.write(LINE_END)?;
                for line in &lines {
                    writer.write(line.as_bytes())?;
                    writer.write(LINE_END)?;
                }
                writer.flush()?;
            }
            None => {
                let mut stdout = std::io::stdout().lock();
                stdout.write(HEADER_LINE)?;
                stdout.write(LINE_END)?;
                for line in &lines {
                    stdout.write(line.as_bytes())?;
                    stdout.write(LINE_END)?;
                }
                stdout.flush()?;
            }
        }
    }

    let duration = now.elapsed()?.as_secs_f32();
    debug!("Time taken: {:.2} seconds", duration);

    Ok(())
}
