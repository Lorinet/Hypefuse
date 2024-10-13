use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, bail};
use log::{error, info};
use toml::{Table, Value};
use serde::{Serialize, Deserialize, Serializer};
use serde::de::StdError;
use serde::ser::SerializeMap;
use toml::value::{Array, Datetime};
use walkdir::WalkDir;

#[derive(Serialize, Debug)]
pub struct ConfigurationRegistry {
    configuration_bases: BTreeMap<String, ConfigurationBase>,
}

impl ConfigurationRegistry {
    pub fn init(&mut self) {
        self.configuration_bases.clear();
    }

    pub fn new() -> ConfigurationRegistry {
        ConfigurationRegistry {
            configuration_bases: BTreeMap::new()
        }
    }

    pub fn get_base(&self, path: &str) -> Option<&ConfigurationBase> {
        self.configuration_bases.get(&path.to_string())
    }

    pub fn get_base_mut(&mut self, path: &str) -> Option<&mut ConfigurationBase> {
        self.configuration_bases.get_mut(&path.to_string())
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

    pub fn get_bases_of_bundle(&self, uuid: &str) -> Vec<&ConfigurationBase> {
        self.configuration_bases.iter().filter(|&(p, b)| p.starts_with(format!("data/{}/config/", uuid).as_str()) && !p.ends_with("/bundle")).map(|(_, b)| b).collect()
    }

    pub fn base_path(uuid: &str, base: &str) -> String {
        format!("data/{}/config/{}", uuid, base)
    }

    pub fn get_base_of_bundle(&self, uuid: &str, base: &str) -> Option<&ConfigurationBase> {
        self.get_base(Self::base_path(uuid, base).as_str())
    }

    pub fn get_base_of_bundle_mut(&mut self, uuid: &str, base: &str) -> Option<&mut ConfigurationBase> {
        self.get_base_mut(Self::base_path(uuid, base).as_str())
    }

    pub fn create_base_of_bundle(&mut self, uuid: &str, base: &str) -> anyhow::Result<()> {
        let path = Self::base_path(uuid, base);
        if let None = self.get_base_of_bundle(uuid, base) {
            self.configuration_bases.insert(path.clone(), ConfigurationBase::new(path.as_str()));
            Ok(())
        } else {
            Err(anyhow!("Base '{}' already exists.", path))
        }
    }

    pub fn delete_base_of_bundle(&mut self, uuid: &str, base: &str) -> anyhow::Result<()> {
        let path = Self::base_path(uuid, base);
        if let None = self.get_base_of_bundle(uuid, base) {
            return Err(anyhow!("Base '{}' does not exist.", Self::base_path(uuid, base)));
        }
        fs::remove_file(path.as_str())?;
        self.configuration_bases.remove(path.as_str());
        Ok(())
    }

    pub fn commit(&self) -> anyhow::Result<()> {
        for (path, base) in self.configuration_bases.iter() {
            base.commit()?;
        }
        Ok(())
    }

    pub fn get_json(&self) -> anyhow::Result<Vec<u8>> {
        let mut bundle_conf = BTreeMap::<String, BTreeMap<String, &ConfigurationBase>>::new();
        for (path, base) in self.configuration_bases.iter() {
            let mut spl = path.split("/").skip(1);
            bundle_conf.entry(spl.next().unwrap().to_string()).or_default().insert(spl.skip(1).next().unwrap().to_string(), &base);
        }
        Ok(serde_json::to_vec(&bundle_conf)?)
    }
}

#[derive(Debug)]
pub struct ConfigurationBase {
    path: String,
    properties: BTreeMap<String, Value>,
}

impl Serialize for ConfigurationBase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut map = serializer.serialize_map(Some(self.properties.len()))?;
        for (k, v) in self.properties.iter() {
            map.serialize_entry(k, v).unwrap();
        }
        map.end()
    }
}

impl ConfigurationBase {
    pub fn new(path: &str) -> ConfigurationBase {
        ConfigurationBase {
            path: path.to_string(),
            properties: BTreeMap::new(),
        }
    }

    pub fn from_file(path: &str) -> anyhow::Result<ConfigurationBase> {
        let file_content = fs::read_to_string(path)?;
        let table = file_content.as_str().parse::<Table>()?;
        let properties = table.into_iter().collect::<BTreeMap<String, Value>>();
        Ok(ConfigurationBase {
            path: path.to_string(),
            properties,
        })
    }

    pub fn is_empty(&self) -> bool {
        return self.properties.len() == 0;
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }

    pub fn to_json(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
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

    pub fn set_json(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        let val: Value = serde_json::from_str::<Value>(value)?.into();
        self.set(key, val)
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
