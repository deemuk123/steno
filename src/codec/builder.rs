use crate::config::{dicts_dir, personal_dict_path};
use crate::dictionary::{load_core, load_file, load_from_dir};
use crate::codec::Codec;

/// Build a Codec with core + all user-installed packs + personal dict.
/// This is the standard way to get a Codec for CLI use.
pub fn build_codec() -> Codec {
    let core = load_core();

    // Load user-installed packs from config dir
    let mut domain = load_from_dir(&dicts_dir());

    // Merge personal dict on top if it exists
    let personal_path = personal_dict_path();
    if personal_path.exists() {
        match load_file(&personal_path) {
            Ok(personal) => {
                for (k, v) in personal.entries {
                    domain.reverse.insert(v.clone(), k.clone());
                    domain.entries.insert(k, v);
                }
            }
            Err(e) => eprintln!("steno: skipping personal dict — {}", e),
        }
    }

    Codec::new(core, domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_codec_works() {
        // Should always succeed even with no user dicts installed
        let codec = build_codec();
        let result = codec.compress("in order to test this").unwrap();
        assert!(result.compressed_len > 0);
    }
}
