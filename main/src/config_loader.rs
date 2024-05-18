use serde::Deserialize;
use std::collections::HashMap;
use fileorama::{Error, Fileorama, LoadStatus, MemoryDriver, MemoryDriverType, Progress};

static CONFIG_LOADER_NAME: &str = "replay_config_loader";

#[derive(Deserialize, Debug)]
struct Config {
    name: String,
    base: String,
    template: Option<bool>,
    metadata: Option<Metadata>,
    pll: Option<Pll>,
    variants: Option<HashMap<String, Variant>>,
    boards: Option<Vec<String>>,
    memory: Option<Memory>,
    config: Option<ConfigOptions>,
    include: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    shortname: String,
    fullname: String,
    manufacturer: String,
    year: String,
    info: String,
    tags: Vec<String>,
    icon: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Pll {
    aux: Option<Aux>,
}

#[derive(Deserialize, Debug)]
struct Aux {
    freq: f64,
    adjustable: bool,
}

#[derive(Deserialize, Debug)]
struct Variant {
    pll: Option<VariantPll>,
    config: Option<HashMap<String, u32>>,
}

#[derive(Deserialize, Debug)]
struct VariantPll {
    sys: Option<Sys>,
    vid: Option<Vid>,
}

#[derive(Deserialize, Debug)]
struct Sys {
    freq: f64,
}

#[derive(Deserialize, Debug)]
struct Vid {
    freq: f64,
}

#[derive(Deserialize, Debug)]
struct Memory {
    verify: bool,
    uploads: Vec<Upload>,
}

#[derive(Deserialize, Debug)]
struct Upload {
    name: String,
    address: u64,
    size: u64,
    swizzle: Option<String>,
    repeat: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct ConfigOptions {
    default: u32,
    options: Vec<OptionEntry>,
}

#[derive(Deserialize, Debug)]
struct OptionEntry {
    name: String,
    bits: String,
    options: Option<Vec<OptionValue>>,
    checkbox: Option<Vec<OptionValue>>,
    menu: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OptionValue {
    name: String,
    value: Option<u32>,
}

#[derive(Debug, Default)]
struct ConfigLoader {
    data: Box<[u8]>,
}

impl MemoryDriver for ConfigLoader {
    fn name(&self) -> &'static str {
        CONFIG_LOADER_NAME
    }

    fn create_instance(&self) -> Box<dyn MemoryDriver> {
        Box::<ConfigLoader>::default()
    }

    fn can_create_from_data(&self, _data: &[u8], file_ext_hint: &str) -> bool {
        match file_ext_hint {
            "json" | "json5" => true, 
            _ => false,
        }
    }

    fn create_from_data(&self, data: Box<[u8]>, file_ext_hint: &str, _driver_data: &Option<Box<[u8]>>) 
        -> Option<Box<dyn MemoryDriver>> {
        match file_ext_hint {
            "json" | "json5" => {
                Some(Box::new(ConfigLoader { data }))
            }
            _ => None,
        }
    }

    fn load(&mut self, _path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
        let slice: &[u8] = &self.data;
        let string_slice: &str = unsafe { std::str::from_utf8_unchecked(slice) };

        let config: Config = match json5::from_str(string_slice) {
            Ok(config) => config, 

            Err(e) => {
                progress.step()?;
                return Err(Error::Generic(format!("Error loading config file: {:?}", e)));
            }
        };
                
        progress.step()?;
        Ok(LoadStatus::Data(fileorama::Fileorama::convert_to_box_u8(Box::new(config))))
    }
}
