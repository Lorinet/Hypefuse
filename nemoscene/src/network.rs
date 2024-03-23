use std::collections::BTreeMap;
use std::process::Command;
use std::thread;
use anyhow::anyhow;
use log::{error, info};
use crate::configuration::ConfigurationRegistry;

pub struct NetworkManager {
    connections: Vec<(String, String)>,
}

impl NetworkManager {
    pub fn new() -> NetworkManager {
        NetworkManager {
            connections: Vec::new(),
        }
    }

    pub fn init(&mut self, configuration: &ConfigurationRegistry) {
        self.connections.clear();
        info!("Setting up WiFi...");
        for network in configuration.get_bases_of_bundle("wifi") {
            let name = network.get_str("name");
            let password = network.get_str("password");
            match (name, password) {
                (Some(name), Some(password)) => { self.connections.push((name, password)); }
                _ => error!("Invalid network configuration"),
            }
        }
        self.connect_all();
    }

    pub fn connect_all(&self) {
        let list = self.connections.clone();
        thread::spawn(move || {
            for (name, password) in list {
                if let Err(error) = Self::connect(name.as_str(), password.as_str()) {
                    error!("Network connection error: {}", error);
                }
            }
            if let Err(error) = Self::hotspot() {
                error!("Could not activate hotspot: {}", error);
            }
        });
    }

    fn hotspot() -> anyhow::Result<()> {
        info!("Activating hotspot as last resort...");
        let status = Command::new("nmcli")
            .arg("device")
            .arg("wifi")
            .arg("hotspot")
            .arg("ssid")
            .arg("LinfinitySmartMirror")
            .arg("password")
            .arg("hypefuse")
            .status()?;
        info!("NetworkManager exit code: {}", status);
        Ok(())
    }

    fn connect(name: &str, password: &str) -> anyhow::Result<()> {
        info!("Connecting to '{}'...", name);
        let status = Command::new("nmcli")
            .arg("device")
            .arg("wifi")
            .arg("connect")
            .arg(name)
            .arg("password")
            .arg(password)
            .status()?;
        info!("NetworkManager exit code: {}", status);
        Ok(())
    }
}