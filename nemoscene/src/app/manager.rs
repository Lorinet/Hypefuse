use std::collections::BTreeMap;
use log::{error, info};
use walkdir::WalkDir;
use crate::app::Bundle;
use crate::configuration::ConfigurationRegistry;

pub struct AppManager {
    bundles: BTreeMap<String, Bundle>,
}

impl AppManager {
    pub fn new() -> AppManager {
        AppManager {
            bundles: BTreeMap::new(),
        }
    }

    pub fn init(&mut self, configuration: &mut ConfigurationRegistry) {
        info!("Loading bundles");
        for folder in WalkDir::new("data/bundles").min_depth(1).max_depth(1) {
            match folder {
                Ok(folder) => if folder.file_type().is_dir() {
                    let path = folder.path().to_str().unwrap();
                    let bundle = Bundle::load_bundle(path);
                    match bundle {
                        Ok(bundle) => {
                            match bundle.load_configuration(configuration) {
                                Ok(()) => { self.bundles.insert(bundle.uuid.clone(), bundle); }
                                Err(error) => error!("Loading bundle configuration failed for bundle '{}': {}", bundle.uuid, error),
                            };
                        }
                        Err(error) => error!("Error loading bundle {}: {}", path, error),
                    };
                },
                Err(error) => error!("Error loading bundle: {}", error),
            };
        }
        info!("Loaded bundles:\n{:#?}", self.bundles);
    }

    pub fn get_bundle(&self, uuid: &str) -> Option<&Bundle> {
        self.bundles.get(uuid)
    }
}