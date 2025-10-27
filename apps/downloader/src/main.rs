use serde::Deserialize;
use tbh_torbox::TorBoxApiConfig;

mod tasks;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    torbox: TorBoxPart,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TorBoxPart {
    api_key: String,
    api_base: String,
    api_version: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let config: Config = tbh_config::load_config_at(
        "config.toml",
        include_str!("../config.default.toml").to_string(),
        true,
    )
    .expect("Unable to load configuration");
    let torbox_config = TorBoxApiConfig::new(
        config.torbox.api_base.to_owned(),
        config.torbox.api_version.to_owned(),
        config.torbox.api_key.to_owned(),
    );
    // Check if the provided API key is valid by doing a sample request to the user endpoint
    tbh_torbox::user::me(&torbox_config, false)
        .await
        .expect("Invalid TorBox account, re-check your API key");
}
