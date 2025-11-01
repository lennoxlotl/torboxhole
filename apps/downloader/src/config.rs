use serde::Deserialize;
use tbh_torbox::TorBoxApiState;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    database_path: String,
    torbox: TorBoxPart,
    directories: DirectoriesPart,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TorBoxPart {
    api_key: String,
    api_base: String,
    api_version: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DirectoriesPart {
    ingest: String,
    download: String,
    output: String,
}

impl Config {
    pub fn torbox(&self) -> &TorBoxPart {
        &self.torbox
    }
    
    pub fn directories(&self) -> &DirectoriesPart {
        &self.directories
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

impl DirectoriesPart {
    pub fn ingest(&self) -> &str {
        &self.ingest
    }
    pub fn download(&self) -> &str {
        &self.download
    }
    pub fn output(&self) -> &str {
        &self.output
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
