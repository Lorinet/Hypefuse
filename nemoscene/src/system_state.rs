use std::{process, thread};
use std::sync::{Arc, Mutex, MutexGuard};
use log::info;
use once_cell::sync::Lazy;
use crate::app::manager::AppManager;
use crate::configuration::ConfigurationRegistry;
use crate::dashboard::{Dashboard, Point};
use crate::network::NetworkManager;
use crate::server::run_server;

pub static SYSTEM_STATE: Lazy<Arc<Mutex<SystemState>>> = Lazy::new(|| {
    let mut system_state = SystemState {
        configuration: ConfigurationRegistry::new(),
        dashboard: Dashboard::new(),
        app_manager: AppManager::new(),
        network_manager: NetworkManager::new(),
    };
    system_state.init();
    Arc::new(Mutex::new(system_state))
});

#[macro_export]
macro_rules! get_system_state {
    () => {
        crate::system_state::SYSTEM_STATE.clone().lock().expect("System state mutex poisoned")
    };
}

pub struct SystemState {
    pub configuration: ConfigurationRegistry,
    pub dashboard: Dashboard,
    pub app_manager: AppManager,
    pub network_manager: NetworkManager,
}

impl SystemState {
    pub fn init(&mut self) {
        info!("Initializing system");
        self.configuration.init();
        self.app_manager.init(&mut self.configuration);
        self.dashboard.init(&self.configuration).expect("Cannot initialize Dashboard");
        self.network_manager.init(&self.configuration);
    }

    pub fn shutdown() -> ! {
        process::exit(0);
        loop {};
    }

}

