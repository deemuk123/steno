use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use crate::build_codec;

// ── Input schemas ────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TextInput {
    #[schemars(description = "The text to process")]
    pub text: String,
}

// ── Server ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StenoServer;

#[tool_router(server_handler)]
impl StenoServer {
    /// Compress text to reduce LLM token usage.
    /// Returns steno-compressed text with a dictionary header for guaranteed round-trip fidelity.
    /// Savings typically 20–70% depending on how much verbose phrasing the text contains.
    #[tool(description = "Compress text to reduce LLM token usage. Returns compressed text with a steno header. Savings typically 20-70%.")]
    fn steno_compress(
        &self,
        Parameters(TextInput { text }): Parameters<TextInput>,
    ) -> String {
        let codec = build_codec();
        match codec.compress(&text) {
            Ok(out) => format!(
                "{}\n\n<!-- steno: {:.1}% saved ({} → {} bytes) -->",
                out.text, out.ratio(), out.original_len, out.compressed_len
            ),
            Err(e) => format!("steno error: {}", e),
        }
    }

    /// Decompress steno-compressed text back to the original.
    /// Requires the same dictionary state that was used to compress.
    #[tool(description = "Decompress steno-compressed text back to the original.")]
    fn steno_decompress(
        &self,
        Parameters(TextInput { text }): Parameters<TextInput>,
    ) -> String {
        let codec = build_codec();
        match codec.decompress(&text) {
            Ok(original) => original,
            Err(e) => format!("steno error: {}", e),
        }
    }

    /// Show compression statistics for text without modifying it.
    /// Useful for deciding whether compression is worth applying.
    #[tool(description = "Show compression statistics for text without modifying it. Returns original size, compressed size, and savings %.")]
    fn steno_stats(
        &self,
        Parameters(TextInput { text }): Parameters<TextInput>,
    ) -> String {
        let codec = build_codec();
        match codec.compress(&text) {
            Ok(out) => format!(
                "original:   {} bytes\ncompressed: {} bytes\nsaved:      {:.1}%",
                out.original_len, out.compressed_len, out.ratio()
            ),
            Err(e) => format!("steno error: {}", e),
        }
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    use rmcp::{ServiceExt, transport::stdio};

    // All logs go to stderr — stdout is reserved for MCP protocol messages
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("steno MCP server starting");

    let service = StenoServer
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("steno MCP server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
