use crate::utils::option_string_to_string;
use timetracker_core::entries::Entry;

#[derive(Clone, Debug)]
pub enum Variable {
    Executable,
    VariableName(String),
}

pub fn combine_variable_names(variables: &[Variable]) -> String {
    let mut key = String::new();
    for (num, variable) in variables.iter().enumerate() {
        let var_name = match variable {
            Variable::Executable => "Executable".to_string(),
            Variable::VariableName(var_name) => var_name.to_string(),
        };

        if var_name.is_empty() {
            continue;
        }

        if num != (variables.len() - 1) {
            key.push_str(&format!("{} ", var_name).to_string());
        } else {
            // Do not add a space for the last variable in the
            // list, so we don't have trailiing whitespace.
            key.push_str(&var_name.clone());
        }
    }
    key
}

/// These variables must be printed in the order that the user wants
/// to use.
///
/// For example each entry may not have the same variable name in
/// var1. In entry A, the var1_name may be PWD and var2_name be USER,
/// and in entry B the var1_name may be USER and var2_name be PWD.
///
/// The user may also want to filter the values and only use a
/// sub-set, such as only use the PWD variable (if it exists), and
/// ignore the USER variable.
pub fn combine_variable_values(entry: &Entry, variables: &[Variable]) -> String {
    let mut key = String::new();

    for (num, variable) in variables.iter().enumerate() {
        let var_value = match variable {
            Variable::Executable => option_string_to_string(&entry.vars.executable),
            Variable::VariableName(var_name) => {
                let var1_name = option_string_to_string(&entry.vars.var1_name);
                let var2_name = option_string_to_string(&entry.vars.var2_name);
                let var3_name = option_string_to_string(&entry.vars.var3_name);
                let var4_name = option_string_to_string(&entry.vars.var4_name);

                if *var_name == var1_name {
                    option_string_to_string(&entry.vars.var1_value)
                } else if *var_name == var2_name {
                    option_string_to_string(&entry.vars.var2_value)
                } else if *var_name == var3_name {
                    option_string_to_string(&entry.vars.var3_value)
                } else if *var_name == var4_name {
                    option_string_to_string(&entry.vars.var4_value)
                } else {
                    "".to_string()
                }
            }
        };

        if var_value.is_empty() {
            continue;
        }

        if num != (variables.len() - 1) {
            key.push_str(&format!("{} ", var_value).to_string());
        } else {
            // Do not add a space for the last variable in the
            // list, so we don't have trailiing whitespace.
            key.push_str(&var_value.clone());
        }
    }
    key
}

pub fn multi_variable_values(entry: &Entry, variables: &[Variable]) -> Vec<String> {
    let mut key = Vec::new();

    for variable in variables.iter() {
        let var_value = match variable {
            Variable::Executable => option_string_to_string(&entry.vars.executable),
            Variable::VariableName(var_name) => {
                let var1_name = option_string_to_string(&entry.vars.var1_name);
                let var2_name = option_string_to_string(&entry.vars.var2_name);
                let var3_name = option_string_to_string(&entry.vars.var3_name);
                let var4_name = option_string_to_string(&entry.vars.var4_name);

                if *var_name == var1_name {
                    option_string_to_string(&entry.vars.var1_value)
                } else if *var_name == var2_name {
                    option_string_to_string(&entry.vars.var2_value)
                } else if *var_name == var3_name {
                    option_string_to_string(&entry.vars.var3_value)
                } else if *var_name == var4_name {
                    option_string_to_string(&entry.vars.var4_value)
                } else {
                    "".to_string()
                }
            }
        };

        if var_value.is_empty() {
            continue;
        }

        key.push(var_value.clone());
    }
    key
}
