use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, bail};
use log::{error, info};
use toml::{Table, Value};
use serde::{Serialize, Deserialize, Serializer};
use serde::de::StdError;
use toml::value::{Array, Datetime};
use walkdir::WalkDir;

pub struct ConfigurationRegistry {
    configuration_bases: BTreeMap<String, ConfigurationBase>,
}

impl ConfigurationRegistry {
    pub fn new() -> ConfigurationRegistry {
        ConfigurationRegistry {
            configuration_bases: BTreeMap::new()
        }
    }

    pub fn get_base(&self, path: &str) -> Option<&ConfigurationBase> {
        self.configuration_bases.get(&path.to_string())
    }

    pub fn load_all(&mut self, path: &str) -> anyhow::Result<()> {
        for base in WalkDir::new(path) {
            match base {
                Ok(base) => if base.file_type().is_file() {
                    info!("{}", base.path().to_str().unwrap());
                    self.load_base(base.path().to_str().unwrap())?
                },
                Err(error) => error!("Error loading configuration base: {}", error),
            };
        }
        Ok(())
    }

    pub fn load_base(&mut self, path: &str) -> anyhow::Result<()> {
        self.configuration_bases.insert(path.to_string(), ConfigurationBase::from_file(path)?);
        Ok(())
    }

    pub fn unload_base(&mut self, path: &str) -> anyhow::Result<()> {
        self.get_base(&path).ok_or(anyhow!("Configuration base not found"))?.commit()?;
        self.configuration_bases.remove(&path.to_string()).unwrap();
        Ok(())
    }

    pub fn get_bases_of(&self, path: &str) -> Vec<&ConfigurationBase> {
        self.configuration_bases.iter().filter(|&(p, b)| p.starts_with(path)).map(|(_, b)| b).collect()
    }

    pub fn commit(&self) -> anyhow::Result<()> {
        for (path, base) in self.configuration_bases.iter() {
            base.commit()?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigurationBase {
    path: String,
    properties: BTreeMap<String, Value>,
}

impl ConfigurationBase {
    pub fn from_file(path: &str) -> anyhow::Result<ConfigurationBase> {
        let file_content = fs::read_to_string(path)?;
        let table = file_content.as_str().parse::<Table>()?;
        let properties = table.into_iter().collect::<BTreeMap<String, Value>>();
        Ok(ConfigurationBase {
            path: path.to_string(),
            properties,
        })
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }

    pub fn get_json(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(val) = self.get(key) {
            return serde_json::to_vec(val).ok();
        }
        None
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        if let Some(val) = self.get(key) {
            if let Value::Integer(val) = val {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        if let Some(val) = self.get(key) {
            if let Value::String(val) = val {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        if let Some(val) = self.get(key) {
            if let Value::Boolean(val) = val {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        if let Some(val) = self.get(key) {
            if let Value::Array(val) = val {
                let res: Result<Vec<String>, _> = val.into_iter().map(|v| (*v).clone().try_into()).collect();
                return res.map_or_else(|e| None, |v| Some(v));
            }
        }
        None
    }

    pub fn get_table(&self, key: &str) -> Option<&Table> {
        if let Some(val) = self.get(key) {
            if let Value::Table(val) = val {
                return Some(val);
            }
        }
        None
    }

    pub fn commit(&self) -> anyhow::Result<()> {
        let toml = toml::ser::to_string(&self.properties)?;
        fs::write(&self.path, toml.as_str())?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: Value) -> anyhow::Result<()> {
        self.properties.insert(key.to_string(), value);
        self.commit()
    }

    pub fn set_i64(&mut self, key: &str, value: i64) -> anyhow::Result<()> {
        self.set(key, Value::Integer(value))
    }

    pub fn set_str(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        self.set(key, Value::String(value.to_string()))
    }

    pub fn set_array<T: TryInto<Value>>(&mut self, key: &str, value: Vec<T>) -> anyhow::Result<()> {
        self.set(key, Value::Array(value.into_iter().map(|val| val.try_into()).collect::<Result<Vec<Value>, _>>().map_err(|e| anyhow!("Value type not supported"))?))
    }
}
