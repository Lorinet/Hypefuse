mod driver;
use std::collections::BTreeMap;
use crate::devices::{Device, Value};

pub struct SHT21 {}

impl SHT21 {
    pub fn new() -> SHT21 {
        SHT21 {}
    }
}

impl Device for SHT21 {
    fn name(&self) -> &str {
        "SHT21"
    }
    fn command(&mut self, command: &str, parameters: &BTreeMap<String, Value>) -> anyhow::Result<Value> {
        Ok(Value::Float(match command {
            "temperature" => driver::SHT21Driver::read_temperature()? as f64,
            "humidity" => driver::SHT21Driver::read_humidity()? as f64,
            _ => anyhow::bail!("Invalid command"),
        }))
    }
}

