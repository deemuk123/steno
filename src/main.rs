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
                Ok(out) => print!("{}", out.text),
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
        },

        Commands::Serve => {
            eprintln!("steno serve: MCP server coming in Phase 3.");
            std::process::exit(1);
        }
    }
}
