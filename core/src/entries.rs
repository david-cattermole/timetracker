use log::debug;
use std::collections::HashMap;

pub use crate::settings::CoreSettings;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RecordRowStatus {
    New,
    Existing,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum EntryStatus {
    Uninitialized = 0,
    Active = 1,
    Idle = 2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EntryVariablesList {
    pub executable: Option<String>,
    pub var1_name: Option<String>,
    pub var2_name: Option<String>,
    pub var3_name: Option<String>,
    pub var4_name: Option<String>,
    pub var5_name: Option<String>,
    pub var1_value: Option<String>,
    pub var2_value: Option<String>,
    pub var3_value: Option<String>,
    pub var4_value: Option<String>,
    pub var5_value: Option<String>,
}

fn set_variable_from_environ_vars(
    variable_name: &Option<String>,
    variable_value: &mut Option<String>,
    environ_vars: &HashMap<String, String>,
) {
    match &variable_name {
        Some(name) => match environ_vars.get(name) {
            Some(value) => {
                debug!("env var name: {:?} value: {:?}", name, value);
                *variable_value = Some(value.to_string());
            }
            None => {
                debug!("env var name {:?} is unavailable.", name);
                *variable_value = None;
            }
        },
        None => *variable_value = None,
    };
}

impl EntryVariablesList {
    pub fn new(
        executable: Option<String>,
        var1_name: Option<String>,
        var2_name: Option<String>,
        var3_name: Option<String>,
        var4_name: Option<String>,
        var5_name: Option<String>,
        var1_value: Option<String>,
        var2_value: Option<String>,
        var3_value: Option<String>,
        var4_value: Option<String>,
        var5_value: Option<String>,
    ) -> EntryVariablesList {
        EntryVariablesList {
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
            var5_value,
        }
    }

    pub fn empty() -> EntryVariablesList {
        EntryVariablesList {
            executable: None,
            var1_name: None,
            var2_name: None,
            var3_name: None,
            var4_name: None,
            var5_name: None,
            var1_value: None,
            var2_value: None,
            var3_value: None,
            var4_value: None,
            var5_value: None,
        }
    }

    pub fn replace_with_environ_vars(&mut self, environ_vars: &HashMap<String, String>) {
        set_variable_from_environ_vars(&self.var1_name, &mut self.var1_value, environ_vars);
        set_variable_from_environ_vars(&self.var2_name, &mut self.var2_value, environ_vars);
        set_variable_from_environ_vars(&self.var3_name, &mut self.var3_value, environ_vars);
        set_variable_from_environ_vars(&self.var4_name, &mut self.var4_value, environ_vars);
        set_variable_from_environ_vars(&self.var5_name, &mut self.var5_value, environ_vars);
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub utc_time_seconds: u64, // Assumed to be UTC time.
    pub duration_seconds: u64,
    pub status: EntryStatus,
    pub vars: EntryVariablesList,
}

impl Entry {
    pub fn new(
        utc_time_seconds: u64,
        duration_seconds: u64,
        status: EntryStatus,
        vars: EntryVariablesList,
    ) -> Entry {
        Entry {
            utc_time_seconds,
            duration_seconds,
            status,
            vars,
        }
    }

    pub fn empty() -> Entry {
        Entry {
            utc_time_seconds: 0_u64,
            duration_seconds: 0_u64,
            status: EntryStatus::Uninitialized,
            vars: EntryVariablesList::empty(),
        }
    }
}

/// Remove duplicate values that repeat for multiple seconds in a row.
///
/// Used to reduce the number of entries and save disk-space and processing
/// time.
pub fn deduplicate_entries(
    last_entry: &Entry,
    entries: &Vec<Entry>,
    record_interval_seconds: u64,
    entries_dedup: &mut Vec<Entry>,
    entry_row_statuses: &mut Vec<RecordRowStatus>,
) {
    let mut new_entries = Vec::<Entry>::new();
    new_entries.push(last_entry.clone());
    for entry in entries {
        new_entries.push(entry.clone());
    }

    if last_entry.status != EntryStatus::Uninitialized {
        entries_dedup.push(last_entry.clone());
        entry_row_statuses.push(RecordRowStatus::Existing);
    }

    let mut last_index = 0;
    let mut last_index_mut = 0;
    let mut last_entry_duration_seconds = last_entry.duration_seconds;
    let mut current_index = 1;
    while current_index < new_entries.len() {
        let last_entry = &new_entries[last_index];
        let current_entry = &new_entries[current_index];

        let last_entry_time = last_entry.utc_time_seconds + last_entry_duration_seconds;
        let current_entry_time = current_entry.utc_time_seconds + current_entry.duration_seconds;
        if last_entry.status != EntryStatus::Uninitialized
            && last_entry_time.abs_diff(current_entry_time) <= record_interval_seconds
            && last_entry.status == current_entry.status
            && last_entry.vars == current_entry.vars
        {
            entries_dedup[last_index_mut].duration_seconds += current_entry.duration_seconds;
            last_entry_duration_seconds = entries_dedup[last_index_mut].duration_seconds;
        } else {
            last_index_mut = entries_dedup.len();
            last_entry_duration_seconds = current_entry.duration_seconds;
            entries_dedup.push(current_entry.clone());
            entry_row_statuses.push(RecordRowStatus::New);

            last_index = current_index;
        }

        current_index += 1;
    }
}

#[cfg(test)]
mod tests {

    use crate::entries::*;
    use anyhow::Result;
    use log::debug;

    #[test]
    fn test_deduplication_all_same_from_scratch() -> Result<()> {
        let mut vars = EntryVariablesList::empty();
        vars.executable = Some("bash".to_string());
        vars.var1_name = Some("project".to_string());
        vars.var2_name = Some("sequence".to_string());
        vars.var3_name = Some("shot".to_string());
        vars.var1_value = Some("project_value".to_string());
        vars.var2_value = Some("sequence_value".to_string());
        vars.var3_value = Some("shot_value".to_string());

        let mut entries_dedup = Vec::<Entry>::new();
        let mut entry_row_statuses = Vec::<RecordRowStatus>::new();

        let last_entry = Entry::empty();

        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new(123456789, 1, EntryStatus::Active, vars.clone()));
        entries.push(Entry::new(123456790, 1, EntryStatus::Active, vars.clone()));
        entries.push(Entry::new(123456791, 1, EntryStatus::Active, vars.clone()));

        let record_interval_seconds = 1;
        deduplicate_entries(
            &last_entry,
            &entries,
            record_interval_seconds,
            &mut entries_dedup,
            &mut entry_row_statuses,
        );

        debug!("entries dedup: {:?}", entries_dedup);
        debug!("entry_row_statuses: {:?}", entry_row_statuses);

        assert_eq!(entries_dedup.len(), 1);
        assert_eq!(entries_dedup[0].duration_seconds, 3);
        assert_eq!(entry_row_statuses.len(), 1);
        assert_eq!(entry_row_statuses[0], RecordRowStatus::New);

        Ok(())
    }

    #[test]
    fn test_deduplication_all_same_with_existing() -> Result<()> {
        let mut vars = EntryVariablesList::empty();
        vars.executable = Some("bash".to_string());
        vars.var1_name = Some("project".to_string());
        vars.var2_name = Some("sequence".to_string());
        vars.var3_name = Some("shot".to_string());
        vars.var1_value = Some("project_value".to_string());
        vars.var2_value = Some("sequence_value".to_string());
        vars.var3_value = Some("shot_value".to_string());

        let mut entries_dedup = Vec::<Entry>::new();
        let mut entry_row_statuses = Vec::<RecordRowStatus>::new();

        let last_entry = Entry::new(123456788, 1, EntryStatus::Active, vars.clone());

        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new(123456789, 1, EntryStatus::Active, vars.clone()));
        entries.push(Entry::new(123456790, 1, EntryStatus::Active, vars.clone()));
        entries.push(Entry::new(123456791, 1, EntryStatus::Active, vars.clone()));

        let record_interval_seconds = 1;
        deduplicate_entries(
            &last_entry,
            &entries,
            record_interval_seconds,
            &mut entries_dedup,
            &mut entry_row_statuses,
        );

        debug!("entries dedup: {:?}", entries_dedup);
        debug!("entry_row_statuses: {:?}", entry_row_statuses);

        assert_eq!(entries_dedup.len(), 1);
        assert_eq!(entry_row_statuses.len(), 1);
        assert_eq!(entries_dedup[0].duration_seconds, 4);
        assert_eq!(entry_row_statuses[0], RecordRowStatus::Existing);

        Ok(())
    }

    #[test]
    fn test_deduplication_some_same_from_scratch() -> Result<()> {
        let mut vars_a = EntryVariablesList::empty();
        vars_a.executable = Some("bash".to_string());
        vars_a.var1_name = Some("project_a".to_string());
        vars_a.var2_name = Some("sequence_a".to_string());
        vars_a.var3_name = Some("shot_a".to_string());
        vars_a.var1_value = Some("project_value_a".to_string());
        vars_a.var2_value = Some("sequence_value_a".to_string());
        vars_a.var3_value = Some("shot_value_a".to_string());

        let mut vars_b = EntryVariablesList::empty();
        vars_b.executable = Some("bash".to_string());
        vars_b.var1_name = Some("project_b".to_string());
        vars_b.var2_name = Some("sequence_b".to_string());
        vars_b.var3_name = Some("shot_b".to_string());
        vars_b.var1_value = Some("project_value_b".to_string());
        vars_b.var2_value = Some("sequence_value_b".to_string());
        vars_b.var3_value = Some("shot_value_b".to_string());

        let mut entries_dedup = Vec::<Entry>::new();
        let mut entry_row_statuses = Vec::<RecordRowStatus>::new();

        let last_entry = Entry::empty();

        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new(
            123456789,
            1,
            EntryStatus::Active,
            vars_a.clone(),
        ));
        entries.push(Entry::new(
            123456790,
            1,
            EntryStatus::Active,
            vars_b.clone(),
        ));
        entries.push(Entry::new(
            123456791,
            1,
            EntryStatus::Active,
            vars_b.clone(),
        ));

        let record_interval_seconds = 1;
        deduplicate_entries(
            &last_entry,
            &entries,
            record_interval_seconds,
            &mut entries_dedup,
            &mut entry_row_statuses,
        );

        debug!("entries dedup: {:?}", entries_dedup);
        debug!("entry_row_statuses: {:?}", entry_row_statuses);

        assert_eq!(entries_dedup.len(), 2);
        assert_eq!(entries_dedup[0].duration_seconds, 1);
        assert_eq!(entries_dedup[1].duration_seconds, 2);
        assert_eq!(entry_row_statuses.len(), 2);
        assert_eq!(entry_row_statuses[0], RecordRowStatus::New);
        assert_eq!(entry_row_statuses[1], RecordRowStatus::New);

        Ok(())
    }

    #[test]
    fn test_deduplication_some_same_with_existing() -> Result<()> {
        let mut vars_a = EntryVariablesList::empty();
        vars_a.executable = Some("bash".to_string());
        vars_a.var1_name = Some("project_a".to_string());
        vars_a.var2_name = Some("sequence_a".to_string());
        vars_a.var3_name = Some("shot_a".to_string());
        vars_a.var1_value = Some("project_value_a".to_string());
        vars_a.var2_value = Some("sequence_value_a".to_string());
        vars_a.var3_value = Some("shot_value_a".to_string());

        let mut vars_b = EntryVariablesList::empty();
        vars_b.executable = Some("bash".to_string());
        vars_b.var1_name = Some("project_b".to_string());
        vars_b.var2_name = Some("sequence_b".to_string());
        vars_b.var3_name = Some("shot_b".to_string());
        vars_b.var1_value = Some("project_value_b".to_string());
        vars_b.var2_value = Some("sequence_value_b".to_string());
        vars_b.var3_value = Some("shot_value_b".to_string());

        let mut entries_dedup = Vec::<Entry>::new();
        let mut entry_row_statuses = Vec::<RecordRowStatus>::new();

        let last_entry = Entry::new(123456788, 1, EntryStatus::Active, vars_a.clone());

        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new(
            123456789,
            1,
            EntryStatus::Active,
            vars_a.clone(),
        ));
        entries.push(Entry::new(
            123456790,
            1,
            EntryStatus::Active,
            vars_b.clone(),
        ));
        entries.push(Entry::new(
            123456791,
            1,
            EntryStatus::Active,
            vars_b.clone(),
        ));

        let record_interval_seconds = 1;
        deduplicate_entries(
            &last_entry,
            &entries,
            record_interval_seconds,
            &mut entries_dedup,
            &mut entry_row_statuses,
        );

        debug!("entries dedup: {:?}", entries_dedup);
        debug!("entry_row_statuses: {:?}", entry_row_statuses);

        assert_eq!(entries_dedup.len(), 2);
        assert_eq!(entry_row_statuses.len(), 2);
        assert_eq!(entries_dedup[0].duration_seconds, 2);
        assert_eq!(entries_dedup[1].duration_seconds, 2);
        assert_eq!(entry_row_statuses[0], RecordRowStatus::Existing);
        assert_eq!(entry_row_statuses[1], RecordRowStatus::New);

        Ok(())
    }

    #[test]
    fn test_deduplication_all_same_with_existing_and_long_timestamp() -> Result<()> {
        let mut vars = EntryVariablesList::empty();
        vars.executable = Some("bash".to_string());
        vars.var1_name = Some("project".to_string());
        vars.var2_name = Some("sequence".to_string());
        vars.var3_name = Some("shot".to_string());
        vars.var1_value = Some("project_value".to_string());
        vars.var2_value = Some("sequence_value".to_string());
        vars.var3_value = Some("shot_value".to_string());

        let mut entries_dedup = Vec::<Entry>::new();
        let mut entry_row_statuses = Vec::<RecordRowStatus>::new();

        let last_entry = Entry::new(123456788, 1, EntryStatus::Active, vars.clone());

        let mut entries = Vec::<Entry>::new();
        entries.push(Entry::new(123456799, 1, EntryStatus::Active, vars.clone()));
        entries.push(Entry::new(123456800, 1, EntryStatus::Active, vars.clone()));
        entries.push(Entry::new(123456801, 1, EntryStatus::Active, vars.clone()));

        let record_interval_seconds = 1;
        deduplicate_entries(
            &last_entry,
            &entries,
            record_interval_seconds,
            &mut entries_dedup,
            &mut entry_row_statuses,
        );

        debug!("entries dedup: {:?}", entries_dedup);
        debug!("entry_row_statuses: {:?}", entry_row_statuses);

        assert_eq!(entries_dedup.len(), 2);
        assert_eq!(entry_row_statuses.len(), 2);
        assert_eq!(entries_dedup[0].duration_seconds, 1);
        assert_eq!(entries_dedup[1].duration_seconds, 3);
        assert_eq!(entry_row_statuses[0], RecordRowStatus::Existing);
        assert_eq!(entry_row_statuses[1], RecordRowStatus::New);

        Ok(())
    }
}
