use crate::get_system_state;
use crate::services::Service;
use log::{error, info, log};
use rppal::gpio::{Gpio, Level, Pin, Trigger};
use std::collections::BTreeMap;
use std::process::Command;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

const PIN_AI: u8 = 27;

pub struct ButtonsService {
    thread: Option<JoinHandle<()>>,
}

impl ButtonsService {
    pub fn new() -> ButtonsService {
        ButtonsService { thread: None }
    }
}

impl Service for ButtonsService {
    fn name(&self) -> &str {
        "buttons_service"
    }

    fn run(&mut self) {
        self.thread = Some(thread::spawn(move || {
            let mut gpio = Gpio::new().unwrap();
            let mut aipin = gpio.get(PIN_AI).unwrap().into_input();
            aipin
                .set_interrupt(Trigger::RisingEdge, Some(Duration::from_millis(5)))
                .unwrap();
            loop {
                if let Ok(Some((pin, ev))) = gpio.poll_interrupts(&[&aipin], false, None) {
                    info!("PIN READ");
                    thread::sleep(Duration::from_millis(500));
                    if pin.read() == Level::High {
                        match pin.pin() {
                            PIN_AI => {
                                match get_system_state!().device_manager.command(
                                    "Gemini",
                                    "voice",
                                    &BTreeMap::new(),
                                ) {
                                    Err(err) => error!("Failed to launch Gemini: {}", err),
                                    Ok(val) => info!("Gemini complete: {:?}", val),
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }));
    }

    fn running(&self) -> bool {
        self.thread.is_some()
    }
}
