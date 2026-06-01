Add Yamaha CK61/CK88 support to this editor, mirroring how the existing devices (re202, ml10x, gr55, minilogue) are wired, and modelling the page after the keyboard's own structure.

THE CODEC
There's a new sibling crate `ck-access-rs` (repo `ragb/ck-access-rs`) with a `ck-wasm` package (wasm-bindgen + tsify, same shape as the others). Wire it in like the rest: add a `{ id: 'ck', repo: 'ragb/ck-access-rs', artifact: 'ck-wasm-pkg' }` target to `scripts/fetch-wasm.mjs`, and load it into `vendor/wasm/` the same way. (For local dev before CI exists, the built package is at `../ck-access-rs/ck-wasm/pkg/` — copy it into `vendor/wasm/ck/`.)

DEVICE MODEL (mimic the hardware)
The CK has two documents:
- System (global): `{ common: SystemCommon, master_eq: MasterEq }`.
- Live Set (one patch, the thing players switch between): `{ common: LiveSetCommon, eq: LiveSetEq, audio_trigger_path: string, zones: Zone[4], parts: Part[3], rotary?: RotarySpeaker, extra_blocks: RawBlock[] }`.
A Live Set = Common settings + 3-band EQ + Rotary Speaker + 4 Zones (master-keyboard MIDI transmit) + 3 Parts (A/B/C — the layered/split sound engines). Build the UI to reflect that: a Live Set screen with tabs/sections for Part A / Part B / Part C, Zone 1–4, Common, EQ, Rotary, plus a separate System screen.

WASM API TO USE (all exported; the .d.ts has full types)
- Typed per-area: `decodeSystemCommon/encodeSystemCommon`, `…MasterEq`, `…LiveSetCommon`, `…LiveSetEq`, `decodeZone/encodeZone`, `decodePart/encodePart`, `decodeRotary/encodeRotary`.
- Whole Live Set: `liveSetFromBlocks(blocks: WasmBlock[]) -> LiveSet` and `liveSetToBlocks(liveSet) -> WasmBlock[]` (`WasmBlock = { address: Uint8Array(3), data: Uint8Array }`).
- Documents -> YAML: `systemToYaml/systemFromYaml`, `liveSetToYaml/liveSetFromYaml` (use these with the existing YamlPanel).
- SysEx builders (hand straight to Web MIDI): `bulkRequest(device, address)`, `bulkDump(device, address, data)`, `parameterChange(device, address, data)`, `parameterRequest(device, address)`, `identityRequest()`, `identifyReply(bytes) -> "CK61"|"CK88"|null`.
- Inbound routing: `classifyInbound(bytes)` -> tagged union (`bulk_dump`/`parameter_change`/`request`/`identity_reply`/`unparseable_sysex`/`non_sysex`).
- Addresses: `systemCommonBase()`, `masterEqBase()`, `liveSetCommonBase()`, `liveSetEqBase()`, `audioTriggerPathBase()`, `zoneBase(0..3)`, `partBase(0..2)`, `bulkHeaderBase()`, `bulkFooterBase()`.
- Help data: `voiceNames(): string[]` (363 voices) + `categoryName(0..9)` for Part voice pickers; `rotarySpecs()` -> `{ key, label, unit: "rpm"|"ratio"|"", min, max, default }[]` for the Rotary sliders.

MIDI FLOW (important — the CK is quirky)
- Read System: `bulkRequest` to `systemCommonBase()` and `masterEqBase()` individually; each returns one Bulk Dump -> decode.
- Read Live Set: the individual Live Set blocks do NOT answer individual requests. Send one `bulkRequest(device, bulkHeaderBase())`; the device streams the whole patch back as a sequence of Bulk Dumps (header, Soundmondo, Common, Audio Trigger, Rotary, EQ, Zone 1–4, Part A–C, footer). Collect them (via `classifyInbound`), drop header/footer, pass the `{address,data}` list to `liveSetFromBlocks`.
- Write Live Set: `liveSetToBlocks(liveSet)`, then send `bulkDump(device, bulkHeaderBase(), [])`, every content block as `bulkDump(device, addr, data)`, then `bulkDump(device, bulkFooterBase(), [])`.
- Live single-param edits (for responsive UI): send `parameterChange(device, address, [value])` where `address = blockBase + offset`. The CK echoes parameter changes it makes, so `classifyInbound` can keep the UI in sync.
- Device number is a user setting (0–15, default 0) — surface it like other devices' channel/id.
- Identity times out on the CK's USB port (it doesn't reply) — make identity best-effort, don't block on it.
- Rotary caveat: rotary edits only persist to the dump after the user Stores the Live Set; note this in the Rotary section's help.

UI / A11Y
Reuse the existing components (EnumSelect, NumberField, RangeField, SwitchField, StringArrayField, YamlPanel, Tabs, SlotPicker, HelpButton/HelpDialog) and the device registry.ts/types.ts pattern. Keep the same accessibility bar as the other pages (ARIA roles, keyboard nav, screen-reader announcements via the announcements store). Part voice selection should use a category -> voice picker driven by `categoryName`/`voiceNames` (Part field `category_voices: number[10]`, `current_category: 0..9`). Rotary sliders should show the documented range/default/unit from `rotarySpecs()` (display the raw 0–127 value; rpm is a documented range, not an exact per-step conversion).

Add a `/ck` route, register the device, and follow the existing pages for layout, dump/sync buttons, and YAML import/export. Match the visual + interaction style of the re202/ml10x pages.
