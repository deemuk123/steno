use clap::{Parser, Subcommand};
use std::io::{self, Read};
use steno::build_codec;

#[derive(Parser)]
#[command(
    name = "steno",
    about = "Compress anything going into your LLM. Less tokens, same meaning.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress text from stdin or a file
    Compress {
        /// Input file (omit to read from stdin)
        file: Option<std::path::PathBuf>,
    },
    /// Decompress steno-compressed text from stdin or a file
    Decompress {
        /// Input file (omit to read from stdin)
        file: Option<std::path::PathBuf>,
    },
    /// Show compression stats without compressing
    Stats {
        /// Input file (omit to read from stdin)
        file: Option<std::path::PathBuf>,
    },
    /// Manage dictionary packs
    Dict {
        #[command(subcommand)]
        action: DictCommands,
    },
    /// Show cumulative compression savings
    Gain,
    /// Analyze text and record phrase frequencies for dictionary suggestions
    Learn {
        /// Input file (omit to read from stdin)
        file: Option<std::path::PathBuf>,
    },
    /// Show frequently-used phrases not yet in any dictionary
    Suggest {
        /// Number of suggestions to show (default: 20)
        #[arg(long, default_value = "20")]
        top: usize,
        /// Minimum occurrences before a phrase is suggested (default: 3)
        #[arg(long, default_value = "3")]
        min: u64,
        /// Auto-add all suggestions to personal dictionary without prompting
        #[arg(long)]
        add: bool,
    },
    /// Start the MCP server (Phase 3)
    Serve,
}

#[derive(Subcommand)]
enum DictCommands {
    /// List installed dictionary packs
    List,
    /// Install a dictionary pack from a file
    Add {
        /// Path to the .toml dictionary pack
        path: std::path::PathBuf,
    },
    /// Remove an installed dictionary pack by name
    Remove {
        /// Pack name (as in the [meta] name field)
        name: String,
    },
    /// Add a single entry to your personal dictionary
    PersonalAdd {
        /// The phrase to compress (e.g. "in order to")
        phrase: String,
        /// The short code to replace it with (e.g. "→")
        code: String,
    },
}

fn read_input(file: Option<std::path::PathBuf>) -> Result<String, String> {
    match file {
        Some(path) => std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {:?}: {}", path, e)),
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("cannot read stdin: {}", e))?;
            Ok(buf)
        }
    }
}

fn die(msg: impl std::fmt::Display) -> ! {
    eprintln!("steno: {}", msg);
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress { file } => {
            let input = read_input(file).unwrap_or_else(|e| die(e));
            let codec = build_codec();
            match codec.compress(&input) {
                Ok(out) => {
                    // Record gain before printing so a pipe doesn't interfere
                    let gain_path = steno::config::gain_path();
                    steno::config::ensure_dirs().ok();
                    let mut stats = steno::gain::GainStats::load(&gain_path);
                    stats.record(out.original_len, out.compressed_len, &gain_path);
                    print!("{}", out.text);
                }
                Err(e) => die(e),
            }
        }

        Commands::Decompress { file } => {
            let input = read_input(file).unwrap_or_else(|e| die(e));
            let codec = build_codec();
            match codec.decompress(&input) {
                Ok(out) => print!("{}", out),
                Err(e) => die(e),
            }
        }

        Commands::Stats { file } => {
            let input = read_input(file).unwrap_or_else(|e| die(e));
            let codec = build_codec();
            match codec.compress(&input) {
                Ok(out) => {
                    eprintln!("Original:   {} bytes", out.original_len);
                    eprintln!("Compressed: {} bytes", out.compressed_len);
                    eprintln!("Saved:      {:.1}%", out.ratio());
                }
                Err(steno::codec::StenoError::AlreadyCompressed) => {
                    eprintln!("This text is already steno-compressed.");
                    eprintln!("Run `steno decompress` first to see original stats.");
                }
                Err(e) => die(e),
            }
        }

        Commands::Dict { action } => match action {
            DictCommands::List => {
                let dir = steno::config::dicts_dir();
                if !dir.exists() {
                    println!("No packs installed. ({})", dir.display());
                    return;
                }
                let mut found = false;
                for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| die(e)).flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                        println!("{}", path.file_name().unwrap().to_string_lossy());
                        found = true;
                    }
                }
                if !found {
                    println!("No packs installed.");
                }
            }

            DictCommands::Add { path } => {
                steno::dictionary::load_file(&path).unwrap_or_else(|e| die(e));
                let dest_dir = steno::config::dicts_dir();
                steno::config::ensure_dirs().unwrap_or_else(|e| die(e));
                let dest = dest_dir.join(path.file_name().unwrap_or_else(|| die("invalid file name")));
                std::fs::copy(&path, &dest).unwrap_or_else(|e| die(e));
                println!("Installed: {}", dest.display());
            }

            DictCommands::Remove { name } => {
                let dir = steno::config::dicts_dir();
                let target = dir.join(format!("{}.toml", name));
                if !target.exists() {
                    die(format!("pack not found: {}", name));
                }
                std::fs::remove_file(&target).unwrap_or_else(|e| die(e));
                println!("Removed: {}", name);
            }

            DictCommands::PersonalAdd { phrase, code } => {
                let path = steno::config::personal_dict_path();
                steno::config::ensure_dirs().unwrap_or_else(|e| die(e));

                // Load existing personal.toml or create a default skeleton
                let mut table: toml::Table = if path.exists() {
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| die(format!("cannot read personal.toml: {}", e)));
                    toml::from_str(&content)
                        .unwrap_or_else(|e| die(format!("invalid personal.toml: {}", e)))
                } else {
                    let mut meta = toml::Table::new();
                    meta.insert("name".into(), toml::Value::String("personal".into()));
                    meta.insert("description".into(), toml::Value::String("Personal custom entries".into()));
                    meta.insert("author".into(), toml::Value::String("user".into()));
                    meta.insert("version".into(), toml::Value::String("1.0.0".into()));
                    meta.insert("language".into(), toml::Value::String("en".into()));
                    let mut t = toml::Table::new();
                    t.insert("meta".into(), toml::Value::Table(meta));
                    t.insert("entries".into(), toml::Value::Table(toml::Table::new()));
                    t
                };

                // Add entry to [entries] table
                let entries = table
                    .entry("entries")
                    .or_insert(toml::Value::Table(toml::Table::new()))
                    .as_table_mut()
                    .unwrap_or_else(|| die("malformed [entries] section in personal.toml"));
                entries.insert(phrase.clone(), toml::Value::String(code.clone()));

                // Write back
                let content = toml::to_string_pretty(&table)
                    .unwrap_or_else(|e| die(format!("cannot serialize personal.toml: {}", e)));
                std::fs::write(&path, content)
                    .unwrap_or_else(|e| die(format!("cannot write personal.toml: {}", e)));

                println!("Added: {:?} → {:?}", phrase, code);
                println!("Personal dict: {}", path.display());
            }
        },

        Commands::Gain => {
            let path = steno::config::gain_path();
            let stats = steno::gain::GainStats::load(&path);
            if stats.total_runs == 0 {
                println!("No compression runs recorded yet. Run `steno compress` first.");
            } else {
                println!("Steno gain report");
                println!("-----------------");
                println!("Runs:      {}", stats.total_runs);
                println!("Original:  {} bytes", stats.total_original_bytes);
                println!("Saved:     {} bytes  ({:.1}%)", stats.bytes_saved(), stats.percent_saved());
            }
        }

        Commands::Learn { file } => {
            let input = read_input(file).unwrap_or_else(|e| die(e));
            let usage_path = steno::config::usage_path();
            steno::config::ensure_dirs().unwrap_or_else(|e| die(e));

            let mut stats = steno::learn::UsageStats::load(&usage_path);
            let before = stats.total_phrases();
            stats.learn_text(&input);
            let after = stats.total_phrases();
            stats.save(&usage_path);

            let new_phrases = after.saturating_sub(before);
            eprintln!("Learned from {} chars of text.", input.len());
            eprintln!("Tracked phrases: {} total (+{} new)", after, new_phrases);
            eprintln!("Run `steno suggest` to see top candidates for your dictionary.");
        }

        Commands::Suggest { top, min, add } => {
            let usage_path = steno::config::usage_path();
            let stats = steno::learn::UsageStats::load(&usage_path);

            if stats.total_phrases() == 0 {
                println!("No usage data yet. Run `steno learn <file>` on your common prompts first.");
                return;
            }

            let codec = build_codec();
            // Build combined dict from codec for filtering
            let combined = {
                let mut d = codec.core_dict.clone();
                for (k, v) in &codec.domain_dict.entries {
                    d.entries.insert(k.clone(), v.clone());
                }
                d
            };

            let suggestions = stats.suggestions(&combined, top, min);

            if suggestions.is_empty() {
                println!("No phrases qualify yet (need {} occurrences minimum).", min);
                println!("Run `steno learn` on more text, or lower --min.");
                return;
            }

            if add {
                // Auto-add all suggestions to personal.toml
                let personal_path = steno::config::personal_dict_path();
                let mut table: toml::Table = if personal_path.exists() {
                    let content = std::fs::read_to_string(&personal_path)
                        .unwrap_or_else(|e| die(format!("cannot read personal.toml: {}", e)));
                    toml::from_str(&content)
                        .unwrap_or_else(|e| die(format!("invalid personal.toml: {}", e)))
                } else {
                    let mut meta = toml::Table::new();
                    meta.insert("name".into(), toml::Value::String("personal".into()));
                    meta.insert("description".into(), toml::Value::String("Personal custom entries".into()));
                    meta.insert("author".into(), toml::Value::String("user".into()));
                    meta.insert("version".into(), toml::Value::String("1.0.0".into()));
                    meta.insert("language".into(), toml::Value::String("en".into()));
                    let mut t = toml::Table::new();
                    t.insert("meta".into(), toml::Value::Table(meta));
                    t.insert("entries".into(), toml::Value::Table(toml::Table::new()));
                    t
                };

                let entries = table
                    .entry("entries")
                    .or_insert(toml::Value::Table(toml::Table::new()))
                    .as_table_mut()
                    .unwrap_or_else(|| die("malformed [entries] in personal.toml"));

                for (phrase, count) in &suggestions {
                    let code = steno::learn::suggest_code(phrase);
                    entries.insert(phrase.clone(), toml::Value::String(code.clone()));
                    println!("Added: {:?} ({} uses) → {:?}", phrase, count, code);
                }

                let content = toml::to_string_pretty(&table)
                    .unwrap_or_else(|e| die(format!("cannot serialize personal.toml: {}", e)));
                std::fs::write(&personal_path, content)
                    .unwrap_or_else(|e| die(format!("cannot write personal.toml: {}", e)));
                println!("\n{} entries added. Review with: steno dict list", suggestions.len());
                println!("Edit codes in: {}", personal_path.display());
            } else {
                // Display-only mode
                println!("{:<40} {:>6}  {:<12}", "Phrase", "Uses", "Proposed code");
                println!("{}", "-".repeat(62));
                for (phrase, count) in &suggestions {
                    let code = steno::learn::suggest_code(phrase);
                    println!("{:<40} {:>6}  {:<12}", phrase, count, code);
                }
                println!("\nTo add all: steno suggest --add");
                println!("To add one: steno dict personal-add \"phrase\" \"code\"");
            }
        }

        Commands::Serve => {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(steno::mcp::server::run())
                .unwrap_or_else(|e| die(e));
        }
    }
}
