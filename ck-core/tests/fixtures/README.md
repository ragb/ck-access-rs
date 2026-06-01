# SysEx fixtures

Each `.syx` file here is a raw byte capture of a single CK SysEx message (Bulk
Dump or Parameter Change), used in `ck-core` round-trip tests.

Filename convention: `<area>_<descriptor>.syx`, e.g.

- `system_common_default.syx`
- `live_set_part-a_init.syx`
- `zone_1_default.syx`

When you add a fixture, add a Rust test asserting that decoding the area from the
dump's payload produces the expected typed value **and** that re-encoding
round-trips byte-exact.

These files are tracked in git. Exploratory captures go in
`tools/explore/captures/` (gitignored) until confirmed and promoted here.

> Nothing here yet — the codec is currently validated only against synthetic,
> defaults-derived payloads. Capturing real device dumps is the next step (see
> `docs/sysex-notes.md` and `tools/explore/`).
