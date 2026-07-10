//! `ck-mcp` — the CK's MCP server.
//!
//! Exposes the CK's parameter metadata, value catalogs, JSON Schemas, and factory
//! defaults to an assistant over the Model Context Protocol, and — given a port —
//! lets it read the Live Set currently loaded on the instrument, edit it, and
//! write it back.
//!
//! ```json
//! { "mcpServers": {
//!     "ck": { "command": "ck-mcp", "args": ["--port", "CK Series"] }
//! }}
//! ```
//!
//! Without `--port` only the offline authoring tools are available. `sync` writes
//! the CK's working memory; committing to a Live Set slot is a separate, explicit
//! `store` argument on that tool. `--read-only` refuses writes altogether.

use clap::Parser;
use midi_access_mcp::MidiConfig;

#[derive(Parser)]
#[command(
    name = "ck-mcp",
    version,
    about = "MCP server for the Yamaha CK61/CK88."
)]
struct Cli {
    /// MIDI port name substring for both directions (e.g. "CK Series").
    #[arg(long)]
    port: Option<String>,
    /// MIDI input port substring (overrides --port).
    #[arg(long)]
    input_port: Option<String>,
    /// MIDI output port substring (overrides --port).
    #[arg(long)]
    output_port: Option<String>,
    /// Device number (0..=15) — must match the CK's MIDI Device Number.
    #[arg(long, default_value_t = 0)]
    device: u8,
    /// Serve the reference tools and `dump`, but refuse to write to the device.
    #[arg(long)]
    read_only: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    midi_access_mcp::serve_with::<ck_core::Ck>(MidiConfig {
        input_port: cli.input_port.or_else(|| cli.port.clone()),
        output_port: cli.output_port.or(cli.port),
        device: cli.device,
        read_only: cli.read_only,
    })
}
