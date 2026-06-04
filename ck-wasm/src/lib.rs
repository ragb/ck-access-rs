//! WASM bindings — exposes `ck-core` to JavaScript / TypeScript.
//!
//! Typed encode/decode accept and return the core models (which render in the
//! generated `.d.ts` via their `tsify` derives). Message builders return raw
//! `Uint8Array`s ready to hand to the Web MIDI API.

use ck_core::address::{
    zone_base_address, Part as PartSlot, AUDIO_TRIGGER_PATH_BASE, BULK_FOOTER_BASE,
    BULK_HEADER_BASE, LIVE_SET_COMMON_BASE, LIVE_SET_EQ_BASE, MASTER_EQ_BASE, SYSTEM_COMMON_BASE,
};
use ck_core::cc;
use ck_core::effects;
use ck_core::yaml::{
    live_set_from_yaml_str, live_set_to_yaml_string, system_from_yaml_str, system_to_yaml_string,
};
use ck_core::{
    classify_inbound as core_classify_inbound, identify_reply, identity_request,
    select_live_set_messages, voices, InboundMessage, LiveSet, LiveSetCommon, LiveSetEq, MasterEq,
    Message, Part, RotarySpeaker, System, SystemCommon, Zone, PAGES, ROTARY_SPECS, SOUNDS_PER_PAGE,
    TOTAL_LIVE_SETS,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

fn addr3(address: &[u8]) -> Result<[u8; 3], JsError> {
    address
        .try_into()
        .map_err(|_| JsError::new("address must be exactly 3 bytes"))
}

// === typed area encode/decode ===

macro_rules! area_bindings {
    ($ty:ty, $dec:ident, $dec_js:literal, $enc:ident, $enc_js:literal) => {
        #[wasm_bindgen(js_name = $dec_js)]
        pub fn $dec(bytes: &[u8]) -> Result<$ty, JsError> {
            <$ty>::from_bytes(bytes).map_err(|e| JsError::new(&e.to_string()))
        }
        #[wasm_bindgen(js_name = $enc_js)]
        pub fn $enc(value: $ty) -> Result<Vec<u8>, JsError> {
            value.to_bytes().map_err(|e| JsError::new(&e.to_string()))
        }
    };
}

area_bindings!(
    SystemCommon,
    decode_system_common,
    "decodeSystemCommon",
    encode_system_common,
    "encodeSystemCommon"
);
area_bindings!(
    MasterEq,
    decode_master_eq,
    "decodeMasterEq",
    encode_master_eq,
    "encodeMasterEq"
);
area_bindings!(
    LiveSetCommon,
    decode_live_set_common,
    "decodeLiveSetCommon",
    encode_live_set_common,
    "encodeLiveSetCommon"
);
area_bindings!(
    LiveSetEq,
    decode_live_set_eq,
    "decodeLiveSetEq",
    encode_live_set_eq,
    "encodeLiveSetEq"
);
area_bindings!(Zone, decode_zone, "decodeZone", encode_zone, "encodeZone");
area_bindings!(Part, decode_part, "decodePart", encode_part, "encodePart");
area_bindings!(
    RotarySpeaker,
    decode_rotary,
    "decodeRotary",
    encode_rotary,
    "encodeRotary"
);

/// Documented Rotary parameter ranges/defaults (rpm/ratio), for display.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmRotarySpec {
    pub key: String,
    pub label: String,
    pub unit: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

#[wasm_bindgen(js_name = rotarySpecs)]
pub fn rotary_specs() -> Vec<WasmRotarySpec> {
    ROTARY_SPECS
        .iter()
        .map(|s| WasmRotarySpec {
            key: s.key.to_string(),
            label: s.label.to_string(),
            unit: s.unit.to_string(),
            min: s.min,
            max: s.max,
            default: s.default,
        })
        .collect()
}

// === document YAML round-trip ===

#[wasm_bindgen(js_name = systemToYaml)]
pub fn system_to_yaml(system: System) -> Result<String, JsError> {
    system_to_yaml_string(&system).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = systemFromYaml)]
pub fn system_from_yaml(text: &str) -> Result<System, JsError> {
    system_from_yaml_str(text).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = liveSetToYaml)]
pub fn live_set_to_yaml(live_set: LiveSet) -> Result<String, JsError> {
    live_set_to_yaml_string(&live_set).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = liveSetFromYaml)]
pub fn live_set_from_yaml(text: &str) -> Result<LiveSet, JsError> {
    live_set_from_yaml_str(text).map_err(|e| JsError::new(&e.to_string()))
}

// === message builders (return wire bytes) ===

#[wasm_bindgen(js_name = parameterChange)]
pub fn parameter_change(device: u8, address: &[u8], data: Vec<u8>) -> Result<Vec<u8>, JsError> {
    Ok(Message::ParameterChange {
        device,
        address: addr3(address)?,
        data,
    }
    .encode())
}

#[wasm_bindgen(js_name = bulkDump)]
pub fn bulk_dump(device: u8, address: &[u8], data: Vec<u8>) -> Result<Vec<u8>, JsError> {
    Ok(Message::BulkDump {
        device,
        address: addr3(address)?,
        data,
    }
    .encode())
}

#[wasm_bindgen(js_name = bulkRequest)]
pub fn bulk_request(device: u8, address: &[u8]) -> Result<Vec<u8>, JsError> {
    Ok(Message::BulkRequest {
        device,
        address: addr3(address)?,
    }
    .encode())
}

#[wasm_bindgen(js_name = parameterRequest)]
pub fn parameter_request(device: u8, address: &[u8]) -> Result<Vec<u8>, JsError> {
    Ok(Message::ParameterRequest {
        device,
        address: addr3(address)?,
    }
    .encode())
}

#[wasm_bindgen(js_name = identityRequest)]
pub fn identity_request_bytes() -> Vec<u8> {
    identity_request().to_vec()
}

/// Returns the detected model name ("CK61"/"CK88") from an Identity Reply, or null.
#[wasm_bindgen(js_name = identifyReply)]
pub fn identify_reply_name(bytes: &[u8]) -> Option<String> {
    identify_reply(bytes).map(|m| m.name().to_string())
}

// === inbound classification ===

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WasmInboundMessage {
    BulkDump {
        device: u8,
        space: String,
        #[tsify(type = "Uint8Array")]
        address: Vec<u8>,
        #[tsify(type = "Uint8Array")]
        data: Vec<u8>,
    },
    ParameterChange {
        device: u8,
        space: String,
        #[tsify(type = "Uint8Array")]
        address: Vec<u8>,
        #[tsify(type = "Uint8Array")]
        data: Vec<u8>,
    },
    Request {
        device: u8,
        #[tsify(type = "Uint8Array")]
        address: Vec<u8>,
        bulk: bool,
    },
    IdentityReply {
        #[tsify(type = "Uint8Array")]
        bytes: Vec<u8>,
    },
    UnparseableSysEx {
        #[tsify(type = "Uint8Array")]
        bytes: Vec<u8>,
        error: String,
    },
    NonSysEx {
        #[tsify(type = "Uint8Array")]
        bytes: Vec<u8>,
    },
}

impl From<InboundMessage> for WasmInboundMessage {
    fn from(m: InboundMessage) -> Self {
        match m {
            InboundMessage::BulkDump {
                device,
                space,
                address,
                data,
            } => Self::BulkDump {
                device,
                space: format!("{space:?}"),
                address: address.to_vec(),
                data,
            },
            InboundMessage::ParameterChange {
                device,
                space,
                address,
                data,
            } => Self::ParameterChange {
                device,
                space: format!("{space:?}"),
                address: address.to_vec(),
                data,
            },
            InboundMessage::Request {
                device,
                address,
                bulk,
            } => Self::Request {
                device,
                address: address.to_vec(),
                bulk,
            },
            InboundMessage::IdentityReply { bytes } => Self::IdentityReply { bytes },
            InboundMessage::UnparseableSysEx { bytes, error } => Self::UnparseableSysEx {
                bytes,
                error: error.to_string(),
            },
            InboundMessage::NonSysEx(bytes) => Self::NonSysEx { bytes },
        }
    }
}

#[wasm_bindgen(js_name = classifyInbound)]
pub fn classify_inbound(bytes: &[u8]) -> WasmInboundMessage {
    core_classify_inbound(bytes).into()
}

// === voices ===

#[wasm_bindgen(js_name = voiceName)]
pub fn voice_name(number: u16) -> Option<String> {
    voices::voice_name(number).map(|s| s.to_string())
}

#[wasm_bindgen(js_name = categoryName)]
pub fn category_name(index: u8) -> Option<String> {
    voices::Category::from_index(index).map(|c| c.name().to_string())
}

/// All 363 voice names in absolute-number order.
#[wasm_bindgen(js_name = voiceNames)]
pub fn voice_names() -> Vec<String> {
    voices::VOICE_NAMES.iter().map(|s| s.to_string()).collect()
}

/// One voice: its absolute number (as stored in `Part.category_voices`) + name.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmVoice {
    pub number: u16,
    pub name: String,
}

/// Inclusive absolute voice-number range `[lo, hi]` owned by category `index`
/// (0..=9), or empty if out of range. `Part.category_voices[index]` holds a
/// value in this range.
#[wasm_bindgen(js_name = categoryVoiceRange)]
pub fn category_voice_range(index: u8) -> Vec<u16> {
    match voices::Category::from_index(index) {
        Some(c) => {
            let (lo, hi) = c.voice_range();
            vec![lo, hi]
        }
        None => Vec::new(),
    }
}

/// Every voice in category `index` (0..=9) as `{ number, name }`, in order.
/// Use this to populate a Part's voice picker — the `number` is exactly what
/// goes into `Part.category_voices[index]`.
#[wasm_bindgen(js_name = voicesInCategory)]
pub fn voices_in_category(index: u8) -> Vec<WasmVoice> {
    match voices::Category::from_index(index) {
        Some(c) => {
            let (lo, hi) = c.voice_range();
            (lo..=hi)
                .filter_map(|n| {
                    voices::voice_name(n).map(|name| WasmVoice {
                        number: n,
                        name: name.to_string(),
                    })
                })
                .collect()
        }
        None => Vec::new(),
    }
}

/// Category index (0..=9) that owns an absolute voice number, or null.
#[wasm_bindgen(js_name = categoryOf)]
pub fn category_of(voice: u16) -> Option<u8> {
    voices::category_of(voice).map(|c| c.index())
}

// === control change descriptions ===

/// Description of one Control Change number.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmCcInfo {
    pub number: u8,
    pub name: String,
    pub affects_tone_generator: bool,
}

/// A controller assign destination (`value` 0..=119 = CC, 120 = USB Audio Volume).
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmAssignTarget {
    pub value: u8,
    pub name: String,
    pub affects_tone_generator: bool,
}

/// Human label for a Control Change number, or null if undocumented.
#[wasm_bindgen(js_name = ccName)]
pub fn cc_name(number: u8) -> Option<String> {
    cc::cc_name(number).map(str::to_string)
}

/// The CK's full Control Change table.
#[wasm_bindgen(js_name = controlChanges)]
pub fn control_changes() -> Vec<WasmCcInfo> {
    cc::CONTROL_CHANGES
        .iter()
        .map(|c| WasmCcInfo {
            number: c.number,
            name: c.name.to_string(),
            affects_tone_generator: c.affects_tone_generator,
        })
        .collect()
}

/// Label for a controller assign value (0..=120), or null if out of range.
#[wasm_bindgen(js_name = assignTargetName)]
pub fn assign_target_name(value: u8) -> Option<String> {
    cc::assign_target_name(value)
}

/// All named assign destinations for a Mod-Wheel / Foot-Pedal dropdown.
#[wasm_bindgen(js_name = assignTargets)]
pub fn assign_targets() -> Vec<WasmAssignTarget> {
    cc::assignable_targets()
        .into_iter()
        .map(|t| WasmAssignTarget {
            value: t.value,
            name: t.name,
            affects_tone_generator: t.affects_tone_generator,
        })
        .collect()
}

// === effect type catalog ===

/// Description of one effect algorithm (`number` is the stored type byte).
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmEffectInfo {
    pub number: u8,
    pub name: String,
}

fn map_effects(list: &[effects::EffectInfo]) -> Vec<WasmEffectInfo> {
    list.iter()
        .map(|e| WasmEffectInfo {
            number: e.number,
            name: e.name.to_string(),
        })
        .collect()
}

/// Part insert-effect algorithms (`Part.effect_1_type` / `effect_2_type`).
#[wasm_bindgen(js_name = partEffects)]
pub fn part_effects() -> Vec<WasmEffectInfo> {
    map_effects(effects::PART_EFFECTS)
}

/// Name of a Part insert-effect type, or null if out of range.
#[wasm_bindgen(js_name = partEffectName)]
pub fn part_effect_name_js(number: u8) -> Option<String> {
    effects::part_effect_name(number).map(str::to_string)
}

/// A/D-input effect algorithms (`LiveSetCommon.ad_effect_1_type` / `ad_effect_2_type`).
#[wasm_bindgen(js_name = adEffects)]
pub fn ad_effects() -> Vec<WasmEffectInfo> {
    map_effects(effects::AD_EFFECTS)
}

/// Name of an A/D-input effect type, or null if out of range.
#[wasm_bindgen(js_name = adEffectName)]
pub fn ad_effect_name_js(number: u8) -> Option<String> {
    effects::ad_effect_name(number).map(str::to_string)
}

// === address helpers ===

#[wasm_bindgen(js_name = systemCommonBase)]
pub fn system_common_base() -> Vec<u8> {
    SYSTEM_COMMON_BASE.to_vec()
}

#[wasm_bindgen(js_name = masterEqBase)]
pub fn master_eq_base() -> Vec<u8> {
    MASTER_EQ_BASE.to_vec()
}

#[wasm_bindgen(js_name = liveSetCommonBase)]
pub fn live_set_common_base() -> Vec<u8> {
    LIVE_SET_COMMON_BASE.to_vec()
}

#[wasm_bindgen(js_name = liveSetEqBase)]
pub fn live_set_eq_base() -> Vec<u8> {
    LIVE_SET_EQ_BASE.to_vec()
}

#[wasm_bindgen(js_name = audioTriggerPathBase)]
pub fn audio_trigger_path_base() -> Vec<u8> {
    AUDIO_TRIGGER_PATH_BASE.to_vec()
}

#[wasm_bindgen(js_name = zoneBase)]
pub fn zone_base(zz: u8) -> Result<Vec<u8>, JsError> {
    zone_base_address(zz)
        .map(|a| a.to_vec())
        .ok_or_else(|| JsError::new("zone index out of range (0..=3)"))
}

#[wasm_bindgen(js_name = partBase)]
pub fn part_base(index: u8) -> Result<Vec<u8>, JsError> {
    PartSlot::from_index(index)
        .map(|p| p.base_address().to_vec())
        .ok_or_else(|| JsError::new("part index out of range (0..=2)"))
}

#[wasm_bindgen(js_name = bulkHeaderBase)]
pub fn bulk_header_base() -> Vec<u8> {
    BULK_HEADER_BASE.to_vec()
}

#[wasm_bindgen(js_name = bulkFooterBase)]
pub fn bulk_footer_base() -> Vec<u8> {
    BULK_FOOTER_BASE.to_vec()
}

// === Live Set selection (Bank Select + Program Change) ===

/// Number of Live Set pages on the CK (20).
#[wasm_bindgen(js_name = liveSetPageCount)]
pub fn live_set_page_count() -> u8 {
    PAGES
}

/// Number of Live Set Sounds per page (8).
#[wasm_bindgen(js_name = liveSetSoundsPerPage)]
pub fn live_set_sounds_per_page() -> u8 {
    SOUNDS_PER_PAGE
}

/// Total user Live Set Sounds on the CK (160).
#[wasm_bindgen(js_name = liveSetTotal)]
pub fn live_set_total() -> u16 {
    TOTAL_LIVE_SETS
}

/// Build the three MIDI frames that switch the CK to a specific Live Set
/// Sound (Bank Select MSB=63, Bank Select LSB=page-1, Program Change=sound-1).
///
/// `channel` is 1-based (1..=16); `page` is 1-based (1..=20); `sound` is
/// 1-based (1..=8). Returns the three frames in send order; the CK only
/// commits the bank latch once the Program Change arrives.
#[wasm_bindgen(js_name = selectLiveSet)]
pub fn select_live_set(channel: u8, page: u8, sound: u8) -> Result<JsValue, JsError> {
    let frames =
        select_live_set_messages(channel, page, sound).map_err(|e| JsError::new(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&frames.to_vec()).map_err(|e| JsError::new(&e.to_string()))
}

// === whole Live Set assembly ===
//
// A full Live Set is read by collecting the Bulk Dump sequence the device
// streams after a Bulk Request to `bulkHeaderBase()`. These helpers convert
// between that `(address, data)` block list and the typed `LiveSet`.

/// One addressed bulk block, as collected from / sent to the device.
#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBlock {
    #[tsify(type = "Uint8Array")]
    pub address: Vec<u8>,
    #[tsify(type = "Uint8Array")]
    pub data: Vec<u8>,
}

/// Assemble a typed `LiveSet` from the collected dump blocks (header/footer ok).
#[wasm_bindgen(js_name = liveSetFromBlocks)]
pub fn live_set_from_blocks(blocks: Vec<WasmBlock>) -> Result<LiveSet, JsError> {
    let pairs: Result<Vec<_>, JsError> = blocks
        .into_iter()
        .map(|b| {
            let addr: [u8; 3] = b
                .address
                .as_slice()
                .try_into()
                .map_err(|_| JsError::new("block address must be 3 bytes"))?;
            Ok((addr, b.data))
        })
        .collect();
    LiveSet::from_blocks(&pairs?).map_err(|e| JsError::new(&e.to_string()))
}

/// Encode a `LiveSet` to its content blocks (no header/footer), device order.
#[wasm_bindgen(js_name = liveSetToBlocks)]
pub fn live_set_to_blocks(live_set: LiveSet) -> Result<Vec<WasmBlock>, JsError> {
    live_set
        .to_blocks()
        .map(|v| {
            v.into_iter()
                .map(|(a, d)| WasmBlock {
                    address: a.to_vec(),
                    data: d,
                })
                .collect()
        })
        .map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use ck_core::{Part, SystemCommon};
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn part_round_trips_via_core() {
        let mut bytes = vec![0u8; 105];
        bytes[0x16] = 0x40; // note shift (centred)
        bytes[0x1C] = 0x40; // pan (centred, v1.10)
        bytes[0x30] = 0x40; // pitch bend range (centred)
        bytes[0x34] = 0x40; // modulation speed (centred)
        let p = Part::from_bytes(&bytes).unwrap();
        assert_eq!(p.to_bytes().unwrap(), bytes);
    }

    #[wasm_bindgen_test]
    fn system_common_decodes() {
        let mut bytes = vec![0u8; 56];
        bytes[0x06] = 0x40;
        bytes[0x07] = 0x40;
        bytes[0x1F] = 0x40;
        bytes[0x0E] = 0x3E;
        bytes[0x11] = 0x40;
        let s = SystemCommon::from_bytes(&bytes).unwrap();
        assert_eq!(s.keyboard_transpose, 0);
    }

    #[wasm_bindgen_test]
    fn voice_lookup() {
        assert_eq!(super::voice_name(0), Some("CFX Stereo".to_string()));
    }
}
