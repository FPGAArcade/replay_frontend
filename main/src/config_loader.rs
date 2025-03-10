/*
///use fileorama::{Driver, DriverType, Error, Fileorama, LoadStatus, Progress};
use serde::Deserialize;
use std::collections::HashMap;

#[allow(dead_code)]
static CONFIG_LOADER_NAME: &str = "replay_config_loader";

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Pll {
    aux: Option<Aux>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Aux {
    freq: f64,
    adjustable: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Variant {
    pll: Option<VariantPll>,
    config: Option<HashMap<String, u32>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct VariantPll {
    sys: Option<Sys>,
    vid: Option<Vid>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Sys {
    freq: f64,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Vid {
    freq: f64,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Memory {
    verify: bool,
    uploads: Vec<Upload>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Upload {
    name: String,
    address: u64,
    size: u64,
    swizzle: Option<String>,
    repeat: Option<bool>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ConfigOptions {
    default: u32,
    options: Vec<OptionEntry>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct OptionEntry {
    name: String,
    bits: String,
    options: Option<Vec<OptionValue>>,
    checkbox: Option<Vec<OptionValue>>,
    menu: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct OptionValue {
    name: String,
    value: Option<u32>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct ConfigLoader {
    data: Box<[u8]>,
}

impl Driver for ConfigLoader {
    fn name(&self) -> &'static str {
        CONFIG_LOADER_NAME
    }

    fn is_remote(&self) -> bool {
        false
    }

    fn supports_url(&self, _path: &str) -> bool {
        false
    }

    fn create_from_url(&self, url: &str) -> Option<DriverType> {
        None
    }

    fn get_directory_list(
        &mut self,
        path: &str,
        progress: &mut Progress,
    ) -> Result<fileorama::FilesDirs, Error> {
        Ok(fileorama::FilesDirs::default())
    }

    fn create_instance(&self) -> DriverType {
        Box::<ConfigLoader>::default()
    }

    fn can_create_from_data(&self, _data: &[u8], file_ext_hint: &str) -> bool {
        matches!(file_ext_hint, "json" | "json5")
    }

    fn create_from_data(
        &self,
        data: Box<[u8]>,
        file_ext_hint: &str,
        _driver_data: &Option<Box<[u8]>>,
    ) -> Option<DriverType> {
        match file_ext_hint {
            "json" | "json5" => Some(Box::new(ConfigLoader { data })),
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
                return Err(Error::Generic(format!(
                    "Error loading config file: {:?}",
                    e
                )));
            }
        };

        progress.step()?;
        Ok(LoadStatus::Data(Fileorama::convert_to_box_u8(Box::new(
            config,
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_amiga_config() {
        let path = std::fs::canonicalize("../data/test_data/amiga_config.json5").unwrap();

        dbg!(&path);

        let vfs = Fileorama::new(1);
        vfs.add_memory_driver(Box::<ConfigLoader>::default());

        let mut io_handler = IoHandler::new(&vfs);
        let handle = io_handler.load_with_driver(&path.to_string_lossy(), CONFIG_LOADER_NAME);

        for _ in 0..100 {
            io_handler.update();

            if let Some(data) = io_handler.get_loaded_as::<Config>(handle) {
                assert_eq!(data.name, "Amiga");
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        panic!("Failed to load config");
    }
}
*/
