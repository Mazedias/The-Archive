use std::{fs, path::Path};
use serde_json::{json, Value};

pub fn generate_file_structure<P: AsRef<Path>>(path: P) -> Result<Value, std::io::Error> {
    let path = path.as_ref();
    let mut entries = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if path.is_dir() {
                entries.push(json!({
                    "name": name,
                    "type": "directory",
                    "contents": generate_file_structure(&path)?
                }));
            } else {
                entries.push(json!({
                    "name": name,
                    "type": "file"
                }));
            }
        }

        Ok(json!({
            "name": path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            "type": "directory",
            "contents": entries
        }))
    } else {
        Ok(json!({
            "name": path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            "type": "file"
        }))
    }
}