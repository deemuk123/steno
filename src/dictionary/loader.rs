use std::path::Path;
use super::types::{DictionaryFile, DictionarySet};

/// Load all .toml dictionary packs from a directory into a DictionarySet.
/// Files that fail to parse are skipped with a warning (fail-safe).
pub fn load_from_dir(dir: &Path) -> DictionarySet {
    let mut set = DictionarySet::new();

    if !dir.exists() {
        return set;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return set,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<DictionaryFile>(&content) {
                Ok(dict) => set.merge(dict),
                Err(e) => eprintln!("steno: skipping {:?} — parse error: {}", path, e),
            },
            Err(e) => eprintln!("steno: skipping {:?} — read error: {}", path, e),
        }
    }

    set
}

/// Load a single dictionary file into a DictionarySet.
pub fn load_file(path: &Path) -> Result<DictionarySet, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {:?}: {}", path, e))?;
    let dict_file: DictionaryFile = toml::from_str(&content)
        .map_err(|e| format!("cannot parse {:?}: {}", path, e))?;
    let mut set = DictionarySet::new();
    set.merge(dict_file);
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_dict(dir: &std::path::Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_load_from_empty_dir() {
        let tmp = std::env::temp_dir().join("steno_test_empty");
        std::fs::create_dir_all(&tmp).unwrap();
        let set = load_from_dir(&tmp);
        assert!(set.entries.is_empty());
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_load_from_dir_with_valid_pack() {
        let tmp = std::env::temp_dir().join("steno_test_valid");
        std::fs::create_dir_all(&tmp).unwrap();
        write_temp_dict(&tmp, "test.toml", r#"
[meta]
name = "test"
description = "test"
author = "test"
version = "0.1.0"
language = "en"

[entries]
"hello world" = "hw"
"#);
        let set = load_from_dir(&tmp);
        assert_eq!(set.compress_lookup("hello world"), Some("hw"));
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_load_from_dir_skips_invalid() {
        let tmp = std::env::temp_dir().join("steno_test_invalid");
        std::fs::create_dir_all(&tmp).unwrap();
        write_temp_dict(&tmp, "bad.toml", "this is not valid toml ][");
        let set = load_from_dir(&tmp);
        assert!(set.entries.is_empty(), "invalid pack should be skipped");
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_load_from_nonexistent_dir() {
        let set = load_from_dir(Path::new("/nonexistent/path/steno_xyz_999"));
        assert!(set.entries.is_empty());
    }
}
