use serde::Deserialize;
use tbh_torbox::TorBoxApiState;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    torbox: TorBoxPart,
    database_path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TorBoxPart {
    api_key: String,
    api_base: String,
    api_version: String,
}

impl Config {
    pub fn torbox(&self) -> &TorBoxPart {
        &self.torbox
    }
    
    pub fn database_path(&self) -> &str {
        &self.database_path
    }
}

impl TorBoxPart {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
    pub fn api_base(&self) -> &str {
        &self.api_base
    }
    pub fn api_version(&self) -> &str {
        &self.api_version
    }
}

impl Into<TorBoxApiState> for &TorBoxPart {
    fn into(self) -> TorBoxApiState {
        TorBoxApiState::new(
            self.api_base.clone(),
            self.api_version.clone(),
            self.api_key.clone(),
        )
    }
}
