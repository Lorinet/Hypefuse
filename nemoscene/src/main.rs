extern crate core;

use std::{env, thread};
use std::time::Duration;
use log::info;
use crate::system_state::SystemState;

pub mod server;
mod system_state;
mod configuration;
mod dashboard;
mod app;

fn main() {

    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Hypefuse [Nemoscene Version 0.1]");
    { get_system_state!(); }
    loop {
         thread::sleep(Duration::from_secs(2));
    }
}
