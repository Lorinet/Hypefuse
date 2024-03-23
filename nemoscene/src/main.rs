extern crate core;

use std::{env, thread};
use std::time::Duration;
use log::info;
use crate::server::run_server;
use crate::system_state::SystemState;

pub mod server;
mod system_state;
mod configuration;
mod dashboard;
mod app;
mod network;

fn main() {

    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Hypefuse [Nemoscene Version 0.1]");
    thread::spawn(run_server);
    { get_system_state!(); }
    loop {
         thread::sleep(Duration::from_secs(2));
    }
}
