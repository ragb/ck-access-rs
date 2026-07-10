# CK61 / CK88 SysEx notes

Protocol reference for `ck-core`. Primary source: the **CK88/CK61 Owner's
Manual** "MIDI Data Format" (§3) and "MIDI Data Table" (the *Data List* section,
printed pages 56–63). Yamaha Group Number **`7F 1C`**, Model ID **`0B`** —
shared by the CK61 and CK88.

> **Verification status.** The framing (Byte Count, checksum), the address map,
> and the System + Live Set area layouts are **device-verified** against a real
> CK88 over USB MIDI: `ck-core/tests/fixtures/` holds captured Bulk Dumps
> (`system_common`, `master_eq`, and a full Live Set), and the tests round-trip
> each byte-exact, including the reserved/undocumented bytes. Engineering-unit
> conversions (dB / Hz / effect-type indices) are still raw bytes pending a
> value-by-value sweep. Identity Reply is **not** emitted by the CK on its USB
> MIDI port (both this CLI and `tools/explore/probe.py` time out) — treat
> `ck identity` as best-effort.

## 1. Message framing

The high nibble of the device byte selects the message kind; the low nibble is
the device number `n` (0..=15).

```
Parameter Change   F0 43 1n 7F 1C 0B  ah am al  dd..dd        F7
Bulk Dump          F0 43 0n 7F 1C bh bl 0B  ah am al dd..dd cc  F7
Bulk Request       F0 43 2n 7F 1C 0B  ah am al                F7
Parameter Request  F0 43 3n 7F 1C 0B  ah am al                F7
```

- `ah am al` — 3-byte parameter address.
- `bh bl` — 14-bit **Byte Count** = `Model ID + Address + Data` length
  (i.e. `data_len + 4`); `bh` = bits 13..7, `bl` = bits 6..0. *(Device-verified:
  a 56-byte System Common payload reports Byte Count 60.)*
- `cc` — **checksum** over `Model ID + Address + Data` (the byte count is **not**
  included), such that `(sum + cc) & 0x7F == 0`. *(Device-verified against
  captured dumps.)* See [`sysex::checksum`](../ck-core/src/sysex.rs).

Request → reply pairing: a **Bulk Request** (`2n`) elicits a **Bulk Dump**
(`0n`) at the same address; a **Parameter Request** (`3n`) elicits a
**Parameter Change** (`1n`).

### Identity

- Request (we send broadcast): `F0 7E 7F 06 01 F7`.
- Reply: `F0 7E 0n 06 02 43 00 41 ff ff vv 00 00 7F F7`, where `ff ff` is the
  device family — **`62 06` = CK61**, **`63 06` = CK88** — and `vv` is the
  firmware version (`(version − 1.0) × 10`).

Worked example (from forum captures), Parts on/off at `50 01 19` = `01`:
`F0 43 10 7F 1C 0B 50 01 19 01 F7`.

## 2. Address map (Parameter Base Address)

| Block               | High | Mid  | Low | Bytes |
|---------------------|------|------|-----|-------|
| System Common       | `20` | `00` | `00`| 56    |
| System Master EQ    | `20` | `40` | `00`| 20    |
| Soundmondo Version  | `00` | `7F` | `00`| 4     |
| Store To Flash      | `0D` | `00` | `00`| —     |
| Bulk Header         | `0E` | `pp` | `0n`| —     |
| Bulk Footer         | `0F` | `pp` | `0n`| —     |
| Live Set Common     | `46` | `00` | `00`| 83    |
| Live Set EQ         | `46` | `40` | `00`| 20    |
| Audio Trigger Path  | `46` | `10` | `00`| 0–255 |
| Live Set (v1.10)    | `46` | `20` | `00`| 24    |
| Zone (zz = 00..=03) | `4A` | `zz` | `00`| 16    |
| Live Set Part (p)   | `50` | `0p` | `00`| 105   |

### Reading / writing a full Live Set

Individual Live Set blocks (`46`/`4A`/`50`) do **not** answer an individual Bulk
Request — only the System blocks (`20`) do. To read a whole patch, send a Bulk
Request to the **Bulk Header** (`0E 00 00`); the CK streams the entire Live Set
back as a sequence of Bulk Dumps:

```text
0E 00 00 (header)  00 7F 00 (Soundmondo ver)  46 00 00 (Common)
46 10 00 (Audio Trigger)  46 20 00 (v1.10 block)  46 40 00 (EQ)
4A 00 00 … 4A 03 00 (Zones 1–4)  50 00 00 … 50 02 00 (Parts A–C)
0F 00 00 (footer)
```

Writing a Live Set replays the same envelope: Bulk Header, every content block,
Bulk Footer. `Audio Trigger Path` is variable length (0 bytes when unset).
[`LiveSet::from_blocks`/`to_blocks`](../ck-core/src/document.rs) convert between
this sequence and the typed document; blocks this crate doesn't model
(`00 7F 00`, `46 20 00`) are preserved verbatim in `extra_blocks`.

### Persisting a Live Set (Store To Flash)

A bulk dump — to the edit buffer (`0E 7F 00`) **or** to a User slot (`0E pp 0n`) —
only writes the CK's working RAM. A slot-addressed dump reads back correctly while
the keyboard stays powered, but is **lost on a power cycle** unless you commit it to
flash. To persist a Live Set, do what the Melas editor does (verified by
intercepting its MIDI output):

1. Dump every Live Set block bracketed to the **destination slot** — Bulk Header
   `0E pp 0n`, the content blocks, Bulk Footer `0F pp 0n` (`pp` = page − 1,
   `0n` = sound − 1), via [`bulk_header_for_slot`/`bulk_footer_for_slot`](../ck-core/src/address.rs).
2. Send the data-less **Store To Flash** commit — a Bulk Dump to `0D 00 00`:

   ```text
   F0 43 00 7F 1C 00 04 0B 0D 00 00 68 F7
   ```

That commit is the panel STORE button's SysEx equivalent; without it the slot dump
never reaches flash. (An earlier read of a one-directional capture mistook the CK's
`0B 10 00` *reply* for the store command — it is not; the real commit is the
manual's `0D 00 00`.) See [`device::Ck::store_to_flash`](../ck-core/src/device.rs)
and [`Ck::store`](../ck-core/src/device.rs), which builds the whole sequence.

## 3. Value encodings

`ck-core::codec` implements the recurring shapes:

- **single byte** — plain `0..=n` value or `0/1` boolean.
- **centred signed** — `value = byte − 0x40` (Transpose, EQ gains, Pan, …).
- **14-bit split** (`Size 2`) — `hi` = bits 13..7, `lo` = bits 6..0 (voice
  numbers, Tempo). Tempo is stored ×10 (`1200` = 120.0 BPM).
- **16-bit nibble-packed** (`Size 4`) — Master Tune; four bytes, one nibble each.
- **ASCII** — Live Set name (15 bytes), Audio Trigger file path (255 bytes).
- **EQ frequency** — every EQ band's frequency (`MasterEq`/`LiveSetEq` low/mid/high,
  and the A/D-input EQ) is a raw index into one shared 1/6-octave frequency table
  ([`crate::eq`](../ck-core/src/eq.rs)). The manual documents only the per-band
  endpoints (Low 32 Hz–2.0 kHz, Mid 100 Hz–10 kHz, High 500 Hz–16 kHz); those six
  land exactly on the standard Yamaha/R20 1/6-octave series, which anchors the
  full table. Endpoints manual-confirmed; intermediate labels are the standard
  series, not individually device-verified. (Gains are `−12..=+12 dB`, already
  typed as `i8`.)
- **effect type** — Part `effect_1/2_type` (`0x00..=0x23`) and A/D `ad_effect_1/2_type`
  (`0x00..=0x22`) are raw indices into the effect-algorithm catalog
  ([`crate::effects`](../ck-core/src/effects.rs)); names come from the *Owner's
  Manual* "Data List" effect-type footnotes (`*2` Part, `*1` A/D). Manually
  transcribed, not yet device-verified.

Field help lives in [`crate::params`](../ck-core/src/params.rs): a runtime
`ParamMeta` catalog (label / group / help / level / catalog-hint) keyed by serde
path (`system.common.master_tune`, `live_set.part.filter_cutoff`, …), so the
editor's `?` buttons read help from the codec instead of a hand-written TS table.

For tools / LLM preset generation: [`crate::catalog`](../ck-core/src/catalog.rs)
emits one JSON bundle (`params` + value catalogs + factory `System`/`LiveSet`
defaults; `ck catalog`), every struct has a factory `Default` + container
`#[serde(default)]` so a *partial* preset deserializes over a complete baseline
(each `parts`/`zones` slot a delta names is merged over that slot's factory
default — so a lone `parts: [{current_category: 1}]` still comes up as Part A
*switched on* — and missing trailing slots are padded, so a delta need only
name the slots and fields it changes),
and [`crate::resolve`](../ck-core/src/resolve.rs) turns value *names* ("Hall
Reverb", "78Rd", "2.0 kHz") into the numeric indices the codec wants (driven by
the `params` catalog-hints; `ck resolve`). Canonical types stay numeric — names
are an input layer only, so the editor and byte codec are unaffected.

Engineering-unit fields whose exact conversion isn't device-confirmed (dB / Hz /
effect-type indices) are kept as **documented raw bytes** so the codec
round-trips exactly; the typed enums cover only the unambiguous cases.

## 4. Firmware compatibility

The base manual is firmware **v1.0**. The **v1.10** supplementary manual adds:

- **Part SW Mode** (Realtime / Next-Key) — a new System Common byte; preserved
  via reserved-byte capture.
- **Pan** per Part (L63–C–R63, centre `0x40`) — **Part block offset `0x1C`**,
  reverse-engineered by capture-and-diff on a real CK and typed as `Part::pan`.
- **Rotary Speaker** (Balance, Stereo/Mono, Speed, Acceleration/Transition for
  Rotary A/B) — the **`46 20 00`** block (24 bytes), reverse-engineered and typed
  as [`RotarySpeaker`](../ck-core/src/rotary.rs). Layout:

  | Offset | Param | Offset | Param |
  |--------|-------|--------|-------|
  | `0x00` | A Balance | `0x0D` | B Balance |
  | `0x01` | A Stereo/Mono | `0x0E` | B Stereo/Mono |
  | `0x02`–`0x05` | A Horn/Rotor Slow, Horn/Rotor Fast | `0x0F`–`0x12` | B Horn/Rotor Slow, Horn/Rotor Fast |
  | `0x06`–`0x09` | A Horn/Rotor Accel, Horn/Rotor Decel | `0x13`–`0x14` | B Horn/Rotor Transition |

  `0x0A`–`0x0C` and `0x15`–`0x17` are reserved (preserved verbatim). Balance
  (`0x00`/`0x0D`) and Transition (`0x13`/`0x14`) offsets are confirmed by their
  v1.10 default values; speed/accel follow the manual's parameter order.
  Documented rpm/ratio ranges are exposed via `ROTARY_SPECS` (the per-byte→rpm
  curve is internal to the device and not published, so values stay raw indices).

  > **Key finding:** Rotary edits only appear in the bulk dump **after you Store
  > the Live Set** — the `46 20 00` block reflects the stored patch, not the live
  > edit buffer. (Pan and most other params update live.)
- **Audio Trigger Play Mode** gains **Hold** (value `3`) — typed.

Every area decoder also tolerates payloads longer than the documented length and
captures any surplus/reserved bytes, so dumps from newer firmware round-trip
byte-exact even where parameters aren't yet typed.
