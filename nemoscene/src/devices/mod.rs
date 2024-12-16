mod sht21;
mod gemini;

use std::collections::BTreeMap;
use log::{error, info};
use serde::{Deserialize, Serialize};
use crate::configuration::ConfigurationRegistry;
use crate::devices::gemini::Gemini;
use crate::devices::sht21::SHT21;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
}

pub trait Device: Send + Sync {
    fn name(&self) -> &str;
    fn command(&mut self, command: &str, parameters: &BTreeMap<String, Value>) -> anyhow::Result<Value>;
}

pub struct DeviceManager {
    devices: BTreeMap<String, Box<dyn Device>>,
}

impl DeviceManager {
    pub fn new() -> DeviceManager {
        DeviceManager {
            devices: BTreeMap::new(),
        }
    }

    pub fn init(&mut self, configuration: &ConfigurationRegistry) {
        self.devices.clear();
        if let Some(devconf) = configuration.get_base_of_bundle("system", "devices") {
            if let Some(true) = devconf.get_bool("sht21") {
                info!("Initializing SHT21...");
                let sht = SHT21::new();
                self.devices.insert(sht.name().to_string(), Box::new(sht));
            }
            info!("Initializing Gemini...");
            let gemini = Gemini::new();
            self.devices.insert(gemini.name().to_string(), Box::new(gemini));
        } else {
            error!("Invalid device configuration");
        }
    }

    pub fn command(&mut self, device: &str, command: &str, parameters: &BTreeMap<String, Value>) -> anyhow::Result<Value> {
        if let Some(dev) = self.devices.get_mut(device) {
            dev.command(command, parameters)
        } else {
            anyhow::bail!("Device {} not found", device);
        }
    }
}