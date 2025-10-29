use crate::config::Config;
use eyre::eyre;
use log::info;
use tbh_torbox::TorBoxApiState;

mod config;
mod processor;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let config: Config = tbh_config::load_config_at(
        "config.toml",
        include_str!("../config.default.toml").to_string(),
        true,
    )
    .unwrap();

    check_torbox_validity(&config.torbox().into())
        .await
        .unwrap();

    tbh_database::create_connection(config.database_path()).unwrap();

    // TODO: app state
}

/// Checks TorBox API configuration by fetching the user profile.
async fn check_torbox_validity(config: &TorBoxApiState) -> eyre::Result<()> {
    info!("Checking provided TorBox API details...");
    tbh_torbox::user::me(&config, false)
        .await
        .map_err(|_| eyre!("Invalid TorBox API key provided, please check your configuration"))?;
    info!("API details are valid");
    Ok(())
}
