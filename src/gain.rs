use serde::{Deserialize, Serialize};
use std::path::Path;

/// Cumulative compression statistics persisted between runs.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GainStats {
    pub total_runs: u64,
    pub total_original_bytes: u64,
    pub total_compressed_bytes: u64,
}

impl GainStats {
    /// Load from disk, returning a default (all zeros) if the file doesn't exist yet.
    pub fn load(path: &Path) -> Self {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        toml::from_str(&raw).unwrap_or_default()
    }

    /// Record a compression result and persist.
    pub fn record(&mut self, original_bytes: usize, compressed_bytes: usize, path: &Path) {
        self.total_runs += 1;
        self.total_original_bytes += original_bytes as u64;
        self.total_compressed_bytes += compressed_bytes as u64;
        // Best-effort write — silently ignore errors so compress never fails
        // just because the stats file isn't writable.
        if let Ok(serialized) = toml::to_string(self) {
            let _ = std::fs::write(path, serialized);
        }
    }

    /// Bytes saved so far.
    pub fn bytes_saved(&self) -> u64 {
        self.total_original_bytes.saturating_sub(self.total_compressed_bytes)
    }

    /// Percentage saved (0.0–100.0), or 0.0 if nothing has been compressed yet.
    pub fn percent_saved(&self) -> f64 {
        if self.total_original_bytes == 0 {
            return 0.0;
        }
        (self.bytes_saved() as f64 / self.total_original_bytes as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_path() -> PathBuf {
        std::env::temp_dir().join(format!("steno-gain-test-{}.toml", std::process::id()))
    }

    #[test]
    fn test_default_zeros() {
        let stats = GainStats::default();
        assert_eq!(stats.total_runs, 0);
        assert_eq!(stats.bytes_saved(), 0);
        assert_eq!(stats.percent_saved(), 0.0);
    }

    #[test]
    fn test_record_and_reload() {
        let path = tmp_path();
        let mut stats = GainStats::load(&path);
        stats.record(100, 80, &path);
        stats.record(200, 150, &path);

        let reloaded = GainStats::load(&path);
        assert_eq!(reloaded.total_runs, 2);
        assert_eq!(reloaded.total_original_bytes, 300);
        assert_eq!(reloaded.total_compressed_bytes, 230);
        assert_eq!(reloaded.bytes_saved(), 70);
        assert!((reloaded.percent_saved() - 23.333).abs() < 0.01);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_missing_file_returns_default() {
        let path = tmp_path(); // does not exist
        let stats = GainStats::load(&path);
        assert_eq!(stats.total_runs, 0);
    }
}
