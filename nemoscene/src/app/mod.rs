use std::collections::BTreeMap;
use std::path::PathBuf;
use anyhow::{anyhow, bail};
use crate::configuration::{ConfigurationBase, ConfigurationRegistry};

pub mod manager;

#[derive(Debug)]
pub struct Bundle {
    pub base_path: String,
    pub uuid: String,
    pub folders: BTreeMap<String, String>,
}

impl Bundle {
    pub fn load_bundle(path: &str) -> anyhow::Result<Bundle> {
        let mut pathbuf = PathBuf::from(path);
        let bundle_info = ConfigurationBase::from_file(pathbuf.join("config").join("bundle").to_str().unwrap())?;
        Ok(Bundle {
            base_path: path.to_string(),
            uuid: bundle_info.get_str("uuid").ok_or(anyhow!("Invalid bundle configuration"))?,
            folders: bundle_info.get_string_array("folders").ok_or(anyhow!("Invalid bundle configuration"))?.iter().map(|f| (f.clone(), pathbuf.join(f).to_str().unwrap().to_string())).collect(),
        })
    }

    pub fn load_configuration(&self, configuration: &mut ConfigurationRegistry) -> anyhow::Result<()> {
        let config_path = PathBuf::from(self.base_path.clone()).join("config");
        configuration.load_all(config_path.to_str().unwrap())
    }
}