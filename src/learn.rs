use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::dictionary::DictionarySet;

/// Tracks n-gram frequencies across text passed through `steno learn`.
/// Persisted to `usage.toml` in the steno config directory.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UsageStats {
    /// phrase (lowercase) → cumulative occurrence count
    pub phrases: HashMap<String, u64>,
}

impl UsageStats {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    /// Best-effort save — never panics, never fails a compress/learn run.
    pub fn save(&self, path: &Path) {
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }

    /// Count all 2–4 word n-grams in `text` and accumulate into phrase counts.
    ///
    /// Filters:
    /// - Each token must be 2+ alphabetic chars (strips leading/trailing punctuation)
    /// - Resulting phrase must be ≥ 10 chars (short phrases save negligible tokens)
    pub fn learn_text(&mut self, text: &str) {
        let words: Vec<String> = text
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| w.len() >= 2 && w.chars().all(|c| c.is_alphabetic() || c == '-' || c == '\''))
            .collect();

        for n in 2..=4usize {
            for window in words.windows(n) {
                let phrase = window.join(" ");
                if phrase.len() < 10 {
                    continue;
                }
                *self.phrases.entry(phrase).or_insert(0) += 1;
            }
        }
    }

    /// Return the top `n` phrases not already in `dict`, with count ≥ `min_count`.
    /// Sorted by count descending (ties broken alphabetically for determinism).
    pub fn suggestions(
        &self,
        dict: &DictionarySet,
        top_n: usize,
        min_count: u64,
    ) -> Vec<(String, u64)> {
        let mut candidates: Vec<_> = self
            .phrases
            .iter()
            .filter(|(phrase, count)| {
                **count >= min_count && !dict.entries.contains_key(*phrase)
            })
            .map(|(p, c)| (p.clone(), *c))
            .collect();
        candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        candidates.truncate(top_n);
        candidates
    }

    /// Total number of unique phrases tracked.
    pub fn total_phrases(&self) -> usize {
        self.phrases.len()
    }
}

/// Propose a short code for a phrase using word initials.
/// "machine learning model" → "mlm"
/// "neural network" → "nn"
pub fn suggest_code(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learn_counts_bigrams() {
        let mut stats = UsageStats::default();
        let text = "neural network processes neural network outputs";
        stats.learn_text(text);
        assert_eq!(stats.phrases.get("neural network"), Some(&2));
    }

    #[test]
    fn test_learn_counts_trigrams() {
        let mut stats = UsageStats::default();
        let text = "the neural network model the neural network model";
        stats.learn_text(text);
        assert_eq!(stats.phrases.get("neural network model"), Some(&2));
    }

    #[test]
    fn test_learn_skips_short_phrases() {
        let mut stats = UsageStats::default();
        // "it is" = 5 chars → filtered out
        let text = "it is it is it is";
        stats.learn_text(text);
        assert!(!stats.phrases.contains_key("it is"));
    }

    #[test]
    fn test_learn_accumulates_across_calls() {
        let mut stats = UsageStats::default();
        stats.learn_text("machine learning model is powerful");
        stats.learn_text("machine learning model is everywhere");
        assert_eq!(stats.phrases.get("machine learning model"), Some(&2));
    }

    #[test]
    fn test_suggestions_excludes_existing_dict() {
        let mut stats = UsageStats::default();
        stats.phrases.insert("neural network".into(), 10);
        stats.phrases.insert("gradient descent".into(), 5);

        let mut dict = DictionarySet::new();
        dict.entries.insert("neural network".into(), "nn".into());
        dict.reverse.insert("nn".into(), "neural network".into());

        let suggestions = stats.suggestions(&dict, 10, 3);
        assert!(suggestions.iter().all(|(p, _)| p != "neural network"));
        assert!(suggestions.iter().any(|(p, _)| p == "gradient descent"));
    }

    #[test]
    fn test_suggestions_sorted_by_count() {
        let mut stats = UsageStats::default();
        stats.phrases.insert("gradient descent".into(), 5);
        stats.phrases.insert("machine learning model".into(), 20);
        stats.phrases.insert("attention mechanism".into(), 12);

        let dict = DictionarySet::new();
        let suggestions = stats.suggestions(&dict, 10, 1);
        assert_eq!(suggestions[0].0, "machine learning model");
        assert_eq!(suggestions[1].0, "attention mechanism");
        assert_eq!(suggestions[2].0, "gradient descent");
    }

    #[test]
    fn test_suggest_code_initials() {
        assert_eq!(suggest_code("machine learning model"), "mlm");
        assert_eq!(suggest_code("neural network"), "nn");
        assert_eq!(suggest_code("in order to"), "iot");
    }
}
