use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::io::ErrorKind;
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::error::ParsingError;

use super::SensorPerset;

pub struct PresetManager {
    file_path: String,
    presets: ArcSwap<HashMap<u8, SensorPerset>>,
}

impl PresetManager {
    pub fn new(file_path: String) -> Self {
        PresetManager {
            file_path,
            presets: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    pub fn load_presets(&self) -> Result<Arc<HashMap<u8, SensorPerset>>, ParsingError>{
        let text = match read_to_string(&self.file_path) {
            Ok(s) => s,
            Err(x) if x.kind() == ErrorKind::NotFound => {
                write(&self.file_path, "[]\n")?;
                let empty = Arc::new(HashMap::new());
                self.presets.store(Arc::clone(&empty));
                return Ok(empty)
            },
            Err(x) => return Err(ParsingError::ReadError(x)),
        };

        let entries: Vec<serde_json::Value> = serde_json::from_str(&text)?;

        let mut map = HashMap::new();

        for (index, value) in entries.into_iter().enumerate() {
            let entry = match serde_json::from_value::<SensorPerset>(value) {
                Ok(e) => e,
                Err(x) => {
                    eprintln!("Skipping preset entry {}: {}", index, x);
                    continue;
                },
            };

            if map.contains_key(&entry.id) {
                eprintln!("Sensor ID: {} has more than one entry", entry.id);
                continue;
            }
            map.insert(entry.id, entry);
        }

        let map = Arc::new(map);
        self.presets.store(Arc::clone(&map));
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    // Spec for load_presets, based on the design decisions in the learning log.
    // Assumed signature: load_presets(&self) -> Result<Arc<HashMap<u8, SensorPerset>>, E>
    // for any error type E. Adjust the calls below if you choose a different one.

    use super::PresetManager;
    use crate::decoders::{DataType, Endian};
    use std::{fs, path::PathBuf};

    // Unique per test and per run, because tests run in parallel.
    fn temp_path(test_name: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join(format!("teledash_{}_{}.json", std::process::id(), test_name));
        let _ = fs::remove_file(&path);
        path
    }

    fn manager_with_file(test_name: &str, contents: &str) -> (PresetManager, PathBuf) {
        let path = temp_path(test_name);
        fs::write(&path, contents).unwrap();
        (PresetManager::new(path.to_string_lossy().into_owned()), path)
    }

    #[test]
    fn loads_real_presets_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/presets.json");
        let manager = PresetManager::new(path.to_string());

        let Ok(presets) = manager.load_presets() else {
            panic!("presets.json should load without errors");
        };

        assert_eq!(presets.len(), 5);
        for id in [42u8, 50, 60, 10, 70] {
            assert!(presets.contains_key(&id), "missing preset id {id}");
        }

        let gps = &presets[&60u8];
        assert_eq!(gps.name, "GPS");
        assert_eq!(gps.fields.len(), 2);
        assert!(matches!(gps.fields[0].data_type, DataType::F32));
        assert!(matches!(gps.fields[0].endian, Endian::LITTLE));

        let pixy = &presets[&70u8];
        assert_eq!(pixy.fields.len(), 6);
        assert!(matches!(pixy.fields[0].data_type, DataType::U8));
        assert_eq!(pixy.fields[0].unit.as_deref(), Some("px"));
        assert_eq!(pixy.fields[4].unit, None); // "unit": null in the file
    }

    #[test]
    fn missing_file_creates_empty_presets_file() {
        let path = temp_path("missing");
        let manager = PresetManager::new(path.to_string_lossy().into_owned());

        let Ok(presets) = manager.load_presets() else {
            panic!("a missing file is not an error");
        };
        assert!(presets.is_empty());

        let written = fs::read_to_string(&path).expect("loader should have created the file");
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&written).expect("created file should hold a JSON array");
        assert!(entries.is_empty());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn invalid_json_is_an_error() {
        let (manager, path) = manager_with_file("invalid_json", r#"[ { "id": 1, "name": "#);
        assert!(manager.load_presets().is_err());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn top_level_not_an_array_is_an_error() {
        let (manager, path) = manager_with_file("not_array", r#"{ "id": 1, "name": "x", "fields": [] }"#);
        assert!(manager.load_presets().is_err());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn duplicate_id_keeps_first_entry() {
        let json = r#"[
            { "id": 1, "name": "first",  "fields": [ { "name": "a", "data_type": "u8",  "endian": "little", "unit": null } ] },
            { "id": 1, "name": "second", "fields": [ { "name": "b", "data_type": "f32", "endian": "big",    "unit": "V"  } ] }
        ]"#;
        let (manager, path) = manager_with_file("duplicate", json);

        let Ok(presets) = manager.load_presets() else {
            panic!("a duplicate id is a warning, not an error");
        };
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[&1u8].name, "first");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn unknown_data_type_skips_only_that_entry() {
        let json = r#"[
            { "id": 1, "name": "good", "fields": [ { "name": "a", "data_type": "u8",  "endian": "little", "unit": null } ] },
            { "id": 2, "name": "bad",  "fields": [ { "name": "b", "data_type": "f24", "endian": "little", "unit": null } ] }
        ]"#;
        let (manager, path) = manager_with_file("unknown_type", json);

        let Ok(presets) = manager.load_presets() else {
            panic!("an unknown data type should skip the entry, not fail the load");
        };
        assert_eq!(presets.len(), 1);
        assert!(presets.contains_key(&1u8));
        assert!(!presets.contains_key(&2u8));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn entry_with_missing_key_is_skipped() {
        let json = r#"[
            { "id": 1, "name": "good", "fields": [ { "name": "a", "data_type": "u8", "endian": "little", "unit": null } ] },
            { "id": 3,                 "fields": [ { "name": "c", "data_type": "u8", "endian": "little", "unit": null } ] }
        ]"#;
        let (manager, path) = manager_with_file("missing_key", json);

        let Ok(presets) = manager.load_presets() else {
            panic!("one malformed entry should not fail the load");
        };
        assert_eq!(presets.len(), 1);
        assert!(!presets.contains_key(&3u8));

        let _ = fs::remove_file(&path);
    }
}
