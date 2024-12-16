mod buttons;

use std::collections::BTreeMap;
use std::thread::JoinHandle;
use log::{error, info};
use serde::{Deserialize, Serialize};
use crate::configuration::ConfigurationRegistry;
use crate::services::buttons::ButtonsService;

pub trait Service: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self);
    fn running(&self) -> bool;
}

pub struct ServiceManager {
    services: BTreeMap<String, Box<dyn Service>>,
}

impl ServiceManager {
    pub fn new() -> ServiceManager {
        ServiceManager {
            services: BTreeMap::new(),
        }
    }

    pub fn init(&mut self, configuration: &ConfigurationRegistry) {
        if let Some(devconf) = configuration.get_base_of_bundle("system", "services") {
            if let Some(true) = devconf.get_bool("buttons") {
                info!("Initializing buttons service...");
                let serv = ButtonsService::new();
                if !self.services.contains_key("buttons_service") || !self.services.get(serv.name()).unwrap().running() {
                    self.services.insert(serv.name().to_string(), Box::new(serv));
                    self.services.get_mut("buttons_service").unwrap().run();
                }
            }
        } else {
            error!("Invalid service configuration");
        }
    }
}