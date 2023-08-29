use crate::settings::CommandArguments;
use crate::settings::ConfigureAppSettings;
use crate::settings::FullConfigurationSettings;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use log::{debug, info};
use std::time::SystemTime;

mod settings;

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("TIMETRACKER_LOG", "warn")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = ConfigureAppSettings::new(&args);
    if settings.is_err() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings.unwrap();
    debug!("Settings validated: {:#?}", settings);

    {
        let now = SystemTime::now();

        let full_settings = FullConfigurationSettings::new(args.defaults);
        if full_settings.is_err() {
            bail!("Configuration structure is invalid: {:?}", full_settings);
        }
        let full_settings = full_settings.unwrap();
        debug!("Configuration structure validated: {:#?}", full_settings);

        let toml = toml::to_string(&full_settings)?;
        info!("Dumping configuration file (in TOML format)...");
        print!("{}", toml);

        // TODO: Get the file name to write out.

        // TODO: Write out the file.

        let duration = now.elapsed()?.as_secs_f32();
        debug!("Time taken: {:.1} seconds", duration);
    }

    Ok(())
}
