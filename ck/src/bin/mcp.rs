//! `ck-mcp` — the CK's MCP server.
//!
//! Exposes the CK's parameter metadata, value catalogs, JSON Schemas, and factory
//! defaults to an assistant over the Model Context Protocol, so a Live Set can be
//! authored *by name* and validated against the real codec before it is ever sent
//! to hardware.
//!
//! Read-only: this never opens a MIDI port. Use the `ck` CLI to `sync` a finished
//! preset (and `--store` to commit it to a slot).
//!
//! Configure it in an MCP client as:
//!
//! ```json
//! { "mcpServers": { "ck": { "command": "ck-mcp" } } }
//! ```

fn main() -> anyhow::Result<()> {
    midi_access_mcp::serve::<ck_core::Ck>()
}
