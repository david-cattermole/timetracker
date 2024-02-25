use crate::main_window::build_ui;
use crate::main_window::GlobalState;
use crate::main_window::GlobalStateRcRefCell;
use crate::settings::CommandArguments;
use crate::settings::PrintGuiAppSettings;

use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::Application;
use log::debug;
use std::cell::RefCell;
use std::rc::Rc;

mod constants;
mod main_window;
mod settings;
mod utils;

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("TIMETRACKER_LOG", "warn")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = PrintGuiAppSettings::new(&args);
    if settings.is_err() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings?;
    debug!("Settings validated: {:#?}", settings);

    let application = Application::builder()
        .application_id(constants::APPLICATION_ID)
        .build();

    let global_state: GlobalStateRcRefCell =
        Rc::new(RefCell::new(GlobalState::new_with_settings(settings)));

    application.connect_activate(clone!(
        @strong global_state =>
            move |app| {
                build_ui(app, global_state.clone())
            }
    ));

    let exit_code = application.run();
    if exit_code != glib::ExitCode::SUCCESS {
        bail!("GtkApplication exited with failure: {:?}", exit_code);
    }

    Ok(())
}
