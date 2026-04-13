// steno — compress anything going into your LLM
pub mod codec;
pub mod config;
pub mod dictionary;
pub mod gain;
pub mod layers;
pub mod learn;
pub mod mcp;

pub use codec::{Codec, CompressedOutput, StenoError, build_codec};
pub use dictionary::DictionarySet;
