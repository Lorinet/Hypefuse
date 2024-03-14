#![feature(c_size_t)]
extern crate core;

use std::env;
use gtk::{prelude::*, Window, WindowType};
use gtk::glib;
use log::info;
use webkit2gtk::WebViewExt;
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
    get_system_state!();
    loop{}
}
