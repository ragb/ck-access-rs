# ck-access-rs

A SysEx codec, CLI, and WASM bindings for the **Yamaha CK61 / CK88** stage
keyboards.

## Crates

| Crate | Purpose | Targets |
|---|---|---|
| `ck-core` | Pure codec: Yamaha SysEx framing, checksum, the 3-byte address map, and typed models for every area (System, Live Set, Zone, Part) with byte-exact round-trip. No MIDI, no file I/O. | native + wasm32 |
| `ck` | CLI over `midir` (`ports`/`identity`/`dump`/`sync`/`show`/`lint`/`diff`/`voices`/`schema`). | native |
| `ck-wasm` | `wasm-bindgen` + `tsify-next` bindings for JS/TS. | wasm32 |

The CK uses Yamaha's address-based parameter protocol: Group Number `7F 1C`,
Model ID `0B`, a 3-byte address, and four message kinds (parameter change, bulk
dump, and their requests). See [docs/sysex-notes.md](docs/sysex-notes.md).

## Status

**⚠️ Not yet hardware-verified.** The protocol is transcribed from the official
*CK88/CK61 Owner's Manual* MIDI Data Table. The codec round-trips byte-exact
against synthetic, defaults-derived payloads (35 passing tests), but **no
message has been exchanged with a real CK yet**. Use
[tools/explore/](tools/explore/) to capture dumps and promote them to fixtures
before trusting writes to hardware. The doc lists the two open unknowns (bulk
checksum coverage; device number).

- **Core** — Yamaha [`Message`](ck-core/src/sysex.rs) (parameter change / bulk
  dump / requests, with checksum + identity), the
  [address map](ck-core/src/address.rs), and typed
  [System](ck-core/src/system.rs) / [Live Set](ck-core/src/live_set.rs) /
  [Zone](ck-core/src/zone.rs) / [Part](ck-core/src/part.rs) areas, plus the full
  [363-voice name table](ck-core/src/voices.rs). Symbolic YAML + generated JSON
  Schemas.
- **CLI** — dump/sync the global **System** or a **Live Set** as one YAML file
  each, plus offline `show`/`lint`/`diff`/`voices`/`schema`.
- **wasm** — typed encode/decode per area, document YAML round-trip, message
  builders, inbound classification, and the voice table — all typed via `tsify`.

## CLI

```
ck ports                                       # list MIDI ports
ck --port CK identity                          # probe model (CK61/CK88) + version
ck --port CK dump --system    -o system.yaml
ck --port CK dump --live-set  -o patch.yaml    # current Live Set (common+eq+4 zones+3 parts)
ck --port CK sync --live-set  -i patch.yaml --verify
ck show  patch.yaml
ck lint  patch.yaml
ck diff  a.yaml b.yaml
ck voices organ                                # browse the voice list
ck schema system|live-set
```

`--device N` must match the keyboard's MIDI Device Number (default 0).

## Development

```
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cd ck-wasm && wasm-pack build --target web --release
```

Regenerate the committed schemas after changing a typed model:

```
cargo run --release --bin ck -- schema system   > schemas/ck-system.schema.json
cargo run --release --bin ck -- schema live-set > schemas/ck-live-set.schema.json
```

Hardware capture / verification scratch lives in [tools/explore/](tools/explore/)
(Python over `mido`). Promote captures from `tools/explore/captures/`
(gitignored) into `ck-core/tests/fixtures/` with a byte-exact round-trip test.

## License

MIT. See [LICENSE](LICENSE).
