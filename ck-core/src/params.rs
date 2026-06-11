//! Editor-facing field metadata: label / group / help / level flag, keyed by a
//! fully-qualified path that matches the serde structure of [`crate::System`] /
//! [`crate::LiveSet`].
//!
//! Path scheme (snake_case field names, same as the typed structs):
//!
//! - `system.common.<field>`        — [`crate::system::SystemCommon`]
//! - `system.master_eq.<field>`     — [`crate::system::MasterEq`]
//! - `live_set.common.<field>`      — [`crate::live_set::LiveSetCommon`]
//! - `live_set.eq.<field>`          — [`crate::live_set::LiveSetEq`]
//! - `live_set.rotary.<field>`      — [`crate::rotary::RotarySpeaker`]
//! - `live_set.zone.<field>`        — [`crate::zone::Zone`] (shared by all 4 zones)
//! - `live_set.part.<field>`        — [`crate::part::Part`] (shared by all 3 parts)
//! - `live_set.audio_trigger_path`  — the Audio Trigger wave-file path
//!
//! `help` text is distilled from each typed area's rustdoc and the *Owner's
//! Manual*. This is the runtime source the editor's `?` help buttons read —
//! the rustdoc→JSDoc in the `.d.ts` is compile-time only and unreadable from
//! running JS.

// The field-metadata *type* now lives in the shared kit (`midi_access_core`);
// this module keeps the CK-specific data table and lookups. Re-exported for
// stability so `ck_core::ParamMeta` and `ck_core::params::ParamMeta` still work.
pub use midi_access_core::meta::{Level, ParamMeta, Params};

const fn p(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Plain,
        kind: None,
        catalog: None,
    }
}

/// Same as [`p`] but flags the field as a `0..N` magnitude.
const fn pl(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Magnitude,
        kind: None,
        catalog: None,
    }
}

/// Same as [`p`] but tags the field with a lookup `catalog` for name resolution.
const fn pc(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
    catalog: &'static str,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Plain,
        kind: None,
        catalog: Some(catalog),
    }
}

/// The full field-metadata catalog, ordered by area then struct-field order.
pub static PARAMS: &[ParamMeta] = &[
    // ===== System / Common =====
    p("system.common.master_tune", "Master tune", "Tune and transpose", "Master tuning of the whole instrument (414.72–466.78 Hz; default A=440)."),
    p("system.common.keyboard_octave_shift", "Octave shift", "Tune and transpose", "Shifts the keyboard up or down in octaves, −3..+3. Not stored per Live Set."),
    p("system.common.keyboard_transpose", "Transpose", "Tune and transpose", "Transposes the keyboard −12..+12 semitones. Not stored per Live Set."),
    p("system.common.controller_reset", "Controller reset", "Output and touch", "What happens to controllers on Live Set change: Hold keeps current values, Reset returns them to defaults."),
    p("system.common.local_control", "Local control", "MIDI", "When off, the keyboard no longer plays the internal tone generator directly (still sends MIDI)."),
    p("system.common.tx_channel", "Transmit channel", "MIDI", "MIDI channel the instrument sends on (1–16, or Off)."),
    p("system.common.rx_channel", "Receive channel", "MIDI", "MIDI channel the instrument responds to (1–16, or All)."),
    p("system.common.midi_control", "MIDI control", "MIDI", "Enables reception of the assignable knob/slider control-change messages."),
    p("system.common.output_gain", "Output gain", "Output and touch", "Analogue output level, −24..+24 dB."),
    p("system.common.touch_curve", "Touch curve", "Output and touch", "Keyboard velocity response: Normal, Soft, Hard, Wide, or Fixed."),
    pl("system.common.fixed_velocity", "Fixed velocity", "Output and touch", "Velocity used for every note when the touch curve is Fixed (1–127)."),
    p("system.common.tx_rx_bank_select", "Bank Select Tx/Rx", "MIDI", "Send and receive Bank Select messages."),
    p("system.common.tx_rx_program_change", "Program Change Tx/Rx", "MIDI", "Send and receive Program Change messages."),
    p("system.common.midi_in_out", "MIDI In/Out", "MIDI", "Enables the 5-pin MIDI In and Out jacks."),
    p("system.common.usb_in_out", "USB In/Out", "MIDI", "Enables MIDI over the USB-to-Host port."),
    p("system.common.value_indication", "Value indication", "Power-on / display", "Show the numeric value as a knob or slider is moved."),
    p("system.common.controller_mode", "Knob pickup", "Output and touch", "How a knob takes over from a stored value: Jump (immediate) or Catch (must pass through)."),
    p("system.common.lcd_switch", "LCD backlight", "Power-on / display", "Turns the screen backlight on or off."),
    p("system.common.lcd_contrast", "LCD contrast", "Power-on / display", "Screen contrast, −10..+10."),
    p("system.common.panel_lock_live_set", "Lock Live Set", "Panel locks", "Prevents the Live Set controls from changing the sound."),
    p("system.common.panel_lock_organ", "Lock Organ", "Panel locks", "Prevents the Organ drawbars/section from changing the sound."),
    p("system.common.panel_lock_filter_eg", "Lock Filter/EG", "Panel locks", "Prevents the Filter and EG knobs from changing the sound."),
    p("system.common.panel_lock_drive_effect", "Lock Drive/Effect", "Panel locks", "Prevents the Drive and Effect knobs from changing the sound."),
    p("system.common.panel_lock_delay_reverb", "Lock Delay/Reverb", "Panel locks", "Prevents the Delay and Reverb knobs from changing the sound."),
    p("system.common.panel_lock_equalizer", "Lock Equalizer", "Panel locks", "Prevents the Equalizer sliders from changing the sound."),
    p("system.common.power_on_page", "Power-on page", "Power-on / display", "Live Set page shown at power-on (1–20)."),
    p("system.common.power_on_sound", "Power-on sound", "Power-on / display", "Live Set sound selected at power-on (1–8)."),
    p("system.common.foot_pedal_1_type", "Foot pedal 1 type", "Foot pedals", "Hardware type of the pedal in jack 1 (FC3A half-damper on/off, FC4A/FC5, or FC7)."),
    p("system.common.foot_pedal_1_live_set", "Foot pedal 1 Live Set", "Foot pedals", "Use foot pedal 1 to step Live Set up, down, or not at all."),
    p("system.common.foot_pedal_2_type", "Foot pedal 2 type", "Foot pedals", "Hardware type of the pedal in jack 2 (FC3A half-damper on/off, FC4A/FC5, or FC7)."),
    p("system.common.foot_pedal_2_live_set", "Foot pedal 2 Live Set", "Foot pedals", "Use foot pedal 2 to step Live Set up, down, or not at all."),
    p("system.common.filter_eg_reset", "Filter/EG reset", "Output and touch", "Reset the Filter/EG knobs to their stored values on Live Set change."),
    p("system.common.effect_on_off_reset", "Effect on/off reset", "Output and touch", "Reset the effect on/off buttons to their stored states on Live Set change."),
    pl("system.common.usb_audio_volume", "USB audio volume", "Audio levels", "Playback level of incoming USB audio (0–127)."),
    pl("system.common.bluetooth_volume", "Bluetooth volume", "Audio levels", "Playback level of incoming Bluetooth audio (0–127)."),
    pl("system.common.ad_volume", "A/D input volume", "Audio levels", "Input level of the rear A/D (mic/line) jacks (0–127)."),
    p("system.common.usb_audio_loopback", "USB audio loopback", "Audio levels", "Send the instrument's own output back over USB audio."),
    p("system.common.speaker_mode", "Speaker mode", "Audio levels", "Internal-speaker voicing: Normal or Table (placed on a flat surface)."),
    p("system.common.speaker_mute", "Speaker mute", "Audio levels", "Whether the speakers mute automatically when headphones are inserted, or manually."),
    // ===== System / Master EQ =====
    p("system.master_eq.low_gain", "Low gain", "Master EQ", "Master EQ low-band gain, −12..+12 dB."),
    pc("system.master_eq.low_freq", "Low frequency", "Master EQ", "Master EQ low-band centre frequency (32 Hz – 2.0 kHz).", "eq_freq"),
    p("system.master_eq.mid_gain", "Mid gain", "Master EQ", "Master EQ mid-band gain, −12..+12 dB."),
    pc("system.master_eq.mid_freq", "Mid frequency", "Master EQ", "Master EQ mid-band centre frequency (100 Hz – 10 kHz).", "eq_freq"),
    p("system.master_eq.high_gain", "High gain", "Master EQ", "Master EQ high-band gain, −12..+12 dB."),
    pc("system.master_eq.high_freq", "High frequency", "Master EQ", "Master EQ high-band centre frequency (500 Hz – 16 kHz).", "eq_freq"),
    // ===== Live Set / Common =====
    p("live_set.common.name", "Live Set name", "Voice", "Name of the Live Set sound (up to 15 characters)."),
    p("live_set.common.live_set_eq_mode", "Live Set EQ", "EQ (3-band)", "Enables the per-Live-Set 3-band EQ."),
    p("live_set.common.master_keyboard_mode", "Master keyboard mode", "Zone basics", "Turns the Live Set into a master-keyboard setup driving external gear via the Zones."),
    p("live_set.common.advanced_zone", "Advanced zone", "Zone basics", "Exposes the full per-Zone transmit settings."),
    p("live_set.common.tempo_x10", "Tempo", "Delay", "Live Set tempo ×10 (e.g. 1200 = 120.0 BPM), used by tempo-synced delay."),
    p("live_set.common.sound_transpose", "Sound transpose", "Voice", "Transposes the Live Set sound −12..+12 semitones."),
    p("live_set.common.layer_split_mode", "Layer / split", "Voice", "How the three parts are laid out: 0 layered (ABC), 1 A/BC, 2 AB/C, 3 A/B/C."),
    p("live_set.common.split_point", "Split point", "Voice", "Split note when splitting into two (note number)."),
    p("live_set.common.split_point_a_b", "Split point A–B", "Voice", "Lower split note when splitting into three."),
    p("live_set.common.split_point_b_c", "Split point B–C", "Voice", "Upper split note when splitting into three."),
    pc("live_set.common.mod_wheel_assign", "Mod wheel assign", "Mod wheel", "Control-change destination the modulation wheel drives (0–119, or 120 = USB audio volume).", "assign_target"),
    pc("live_set.common.foot_pedal_1_assign", "Foot pedal 1 assign", "Foot pedal 1", "Control-change destination foot pedal 1 drives (0–119, or 120 = USB audio volume).", "assign_target"),
    p("live_set.common.foot_pedal_1_limit_low", "Foot pedal 1 min", "Foot pedal 1", "Value foot pedal 1 sends at the heel position (0–127)."),
    p("live_set.common.foot_pedal_1_limit_high", "Foot pedal 1 max", "Foot pedal 1", "Value foot pedal 1 sends at the toe position (0–127)."),
    pc("live_set.common.foot_pedal_2_assign", "Foot pedal 2 assign", "Foot pedal 2", "Control-change destination foot pedal 2 drives (0–119, or 120 = USB audio volume).", "assign_target"),
    p("live_set.common.foot_pedal_2_limit_low", "Foot pedal 2 min", "Foot pedal 2", "Value foot pedal 2 sends at the heel position (0–127)."),
    p("live_set.common.foot_pedal_2_limit_high", "Foot pedal 2 max", "Foot pedal 2", "Value foot pedal 2 sends at the toe position (0–127)."),
    p("live_set.common.delay_switch", "Delay", "Delay", "Master delay on/off."),
    p("live_set.common.delay_type", "Delay type", "Delay", "Delay algorithm: Digital, Analog, Cross, or Tempo (tempo-synced)."),
    pl("live_set.common.delay_depth", "Delay depth", "Delay", "Delay send/mix amount (0–127)."),
    p("live_set.common.delay_time", "Delay time", "Delay", "Delay time (0–127), or note value when the type is Tempo."),
    p("live_set.common.delay_tempo_time", "Delay note value", "Delay", "Tempo-delay note division (used when delay type is Tempo)."),
    p("live_set.common.reverb_switch", "Reverb", "Reverb", "Master reverb on/off."),
    p("live_set.common.reverb_type", "Reverb type", "Reverb", "Reverb algorithm: Hall, Room, or Plate."),
    pl("live_set.common.reverb_depth", "Reverb depth", "Reverb", "Reverb send/mix amount (0–127)."),
    p("live_set.common.rotary_speed", "Rotary speed", "Reverb", "Rotary speaker speed: Slow or Fast."),
    p("live_set.common.rotary_stop", "Rotary stop", "Reverb", "Stops the rotary speaker (brake)."),
    p("live_set.common.audio_trigger_switch", "Audio trigger", "Audio trigger", "Enables playback of the assigned audio file from the keyboard."),
    pl("live_set.common.audio_trigger_volume", "Audio trigger volume", "Audio trigger", "Playback level of the audio-trigger file (0–127)."),
    p("live_set.common.audio_trigger_key", "Audio trigger key", "Audio trigger", "Which held key triggers the file: Lowest or Highest."),
    p("live_set.common.audio_trigger_play_mode", "Audio trigger mode", "Audio trigger", "Playback behaviour: One Shot, Play/Stop, Play/Pause, or Hold (v1.10)."),
    pc("live_set.common.ad_eq_low_freq", "A/D EQ low freq", "EQ (3-band)", "A/D-input EQ low-band centre frequency (32 Hz – 2.0 kHz).", "eq_freq"),
    p("live_set.common.ad_eq_low_gain", "A/D EQ low gain", "EQ (3-band)", "A/D-input EQ low-band gain, −12..+12 dB."),
    pc("live_set.common.ad_eq_mid_freq", "A/D EQ mid freq", "EQ (3-band)", "A/D-input EQ mid-band centre frequency (100 Hz – 10 kHz).", "eq_freq"),
    p("live_set.common.ad_eq_mid_gain", "A/D EQ mid gain", "EQ (3-band)", "A/D-input EQ mid-band gain, −12..+12 dB."),
    pc("live_set.common.ad_eq_high_freq", "A/D EQ high freq", "EQ (3-band)", "A/D-input EQ high-band centre frequency (500 Hz – 16 kHz).", "eq_freq"),
    p("live_set.common.ad_eq_high_gain", "A/D EQ high gain", "EQ (3-band)", "A/D-input EQ high-band gain, −12..+12 dB."),
    p("live_set.common.ad_noise_gate_switch", "Noise gate", "Noise gate", "Enables the A/D-input noise gate."),
    p("live_set.common.ad_noise_gate_threshold", "Gate threshold", "Noise gate", "A/D noise-gate threshold (−73 .. −30 dB)."),
    pc("live_set.common.ad_effect_1_type", "A/D effect 1 type", "Insert effects", "Algorithm for the A/D-input insert effect 1.", "ad_effects"),
    pl("live_set.common.ad_effect_1_depth", "A/D effect 1 depth", "Insert effects", "Depth/mix of A/D insert effect 1 (0–127)."),
    pl("live_set.common.ad_effect_1_rate", "A/D effect 1 rate", "Insert effects", "Rate of A/D insert effect 1 (0–127)."),
    pc("live_set.common.ad_effect_2_type", "A/D effect 2 type", "Insert effects", "Algorithm for the A/D-input insert effect 2.", "ad_effects"),
    pl("live_set.common.ad_effect_2_depth", "A/D effect 2 depth", "Insert effects", "Depth/mix of A/D insert effect 2 (0–127)."),
    pl("live_set.common.ad_effect_2_rate", "A/D effect 2 rate", "Insert effects", "Rate of A/D insert effect 2 (0–127)."),
    pl("live_set.common.ad_volume", "A/D level", "Insert effects", "A/D-input level within the Live Set (0–127)."),
    // ===== Live Set / EQ =====
    p("live_set.eq.low_gain", "Low gain", "EQ (3-band)", "Live Set EQ low-band gain, −12..+12 dB."),
    pc("live_set.eq.low_freq", "Low frequency", "EQ (3-band)", "Live Set EQ low-band centre frequency (32 Hz – 2.0 kHz).", "eq_freq"),
    p("live_set.eq.mid_gain", "Mid gain", "EQ (3-band)", "Live Set EQ mid-band gain, −12..+12 dB."),
    pc("live_set.eq.mid_freq", "Mid frequency", "EQ (3-band)", "Live Set EQ mid-band centre frequency (100 Hz – 10 kHz).", "eq_freq"),
    p("live_set.eq.high_gain", "High gain", "EQ (3-band)", "Live Set EQ high-band gain, −12..+12 dB."),
    pc("live_set.eq.high_freq", "High frequency", "EQ (3-band)", "Live Set EQ high-band centre frequency (500 Hz – 16 kHz).", "eq_freq"),
    // ===== Live Set / Audio Trigger path =====
    p("live_set.audio_trigger_path", "Audio trigger file", "Audio trigger", "Path of the audio file played by the audio trigger."),
    // ===== Live Set / Rotary =====
    p("live_set.rotary.a_balance", "Rotary A balance", "Rotary A", "Balance between horn (treble) and rotor (bass) for Rotary A (R63>H .. R=H .. R<H63)."),
    p("live_set.rotary.a_stereo_mono", "Rotary A stereo/mono", "Rotary A", "Stereo or mono output for Rotary A."),
    p("live_set.rotary.a_horn_slow", "Rotary A horn slow", "Rotary A", "Horn slow speed for Rotary A (23.0–89.6 rpm)."),
    p("live_set.rotary.a_rotor_slow", "Rotary A rotor slow", "Rotary A", "Rotor slow speed for Rotary A (22.7–88.3 rpm)."),
    p("live_set.rotary.a_horn_fast", "Rotary A horn fast", "Rotary A", "Horn fast speed for Rotary A (209.4–817.6 rpm)."),
    p("live_set.rotary.a_rotor_fast", "Rotary A rotor fast", "Rotary A", "Rotor fast speed for Rotary A (189.3–736.8 rpm)."),
    p("live_set.rotary.a_horn_acceleration", "Rotary A horn accel", "Rotary A", "How fast the Rotary A horn speeds up (0.21–2.00)."),
    p("live_set.rotary.a_rotor_acceleration", "Rotary A rotor accel", "Rotary A", "How fast the Rotary A rotor speeds up (0.21–2.00)."),
    p("live_set.rotary.a_horn_deceleration", "Rotary A horn decel", "Rotary A", "How fast the Rotary A horn slows down (0.21–2.00)."),
    p("live_set.rotary.a_rotor_deceleration", "Rotary A rotor decel", "Rotary A", "How fast the Rotary A rotor slows down (0.21–2.00)."),
    p("live_set.rotary.b_balance", "Rotary B balance", "Rotary B", "Balance between horn (treble) and rotor (bass) for Rotary B (R63>H .. R=H .. R<H63)."),
    p("live_set.rotary.b_stereo_mono", "Rotary B stereo/mono", "Rotary B", "Stereo or mono output for Rotary B."),
    p("live_set.rotary.b_horn_slow", "Rotary B horn slow", "Rotary B", "Horn slow speed for Rotary B (2.5–159.0 rpm)."),
    p("live_set.rotary.b_rotor_slow", "Rotary B rotor slow", "Rotary B", "Rotor slow speed for Rotary B (2.5–159.0 rpm)."),
    p("live_set.rotary.b_horn_fast", "Rotary B horn fast", "Rotary B", "Horn fast speed for Rotary B (161.5–2382 rpm)."),
    p("live_set.rotary.b_rotor_fast", "Rotary B rotor fast", "Rotary B", "Rotor fast speed for Rotary B (161.5–2382 rpm)."),
    p("live_set.rotary.b_horn_transition", "Rotary B horn transition", "Rotary B", "Horn acceleration/deceleration amount for Rotary B (0–127)."),
    p("live_set.rotary.b_rotor_transition", "Rotary B rotor transition", "Rotary B", "Rotor acceleration/deceleration amount for Rotary B (0–127)."),
    // ===== Live Set / Zone =====
    p("live_set.zone.zone_switch", "Zone on", "Zone basics", "Enables this Zone's MIDI transmission."),
    p("live_set.zone.transmit_channel", "Transmit channel", "Zone basics", "MIDI channel this Zone transmits on (1–16)."),
    p("live_set.zone.transpose_octave", "Octave", "Range and tuning", "Octave shift applied to this Zone's transmitted notes, −3..+3."),
    p("live_set.zone.transpose_semitone", "Transpose", "Range and tuning", "Semitone shift applied to this Zone's transmitted notes, −11..+11."),
    p("live_set.zone.note_limit_low", "Note limit low", "Range and tuning", "Lowest key this Zone transmits (C-2 .. G8)."),
    p("live_set.zone.note_limit_high", "Note limit high", "Range and tuning", "Highest key this Zone transmits (C-2 .. G8)."),
    pl("live_set.zone.midi_volume", "MIDI volume", "Mix", "Channel Volume (CC 7) this Zone sends (0–127)."),
    p("live_set.zone.midi_pan", "MIDI pan", "Mix", "Pan (CC 10) this Zone sends, L64 .. C .. R63."),
    p("live_set.zone.midi_bank_msb", "Bank MSB", "Bank and program", "Bank Select MSB this Zone sends (0–127)."),
    p("live_set.zone.midi_bank_lsb", "Bank LSB", "Bank and program", "Bank Select LSB this Zone sends (0–127)."),
    p("live_set.zone.midi_program", "Program number", "Bank and program", "Program Change this Zone sends (program 1–128)."),
    p("live_set.zone.transmit.bank_select", "Send Bank Select", "Transmit on Live Set load", "Send Bank Select MSB+LSB when this Live Set loads."),
    p("live_set.zone.transmit.program_change", "Send Program Change", "Transmit on Live Set load", "Send Program Change when this Live Set loads."),
    p("live_set.zone.transmit.volume", "Send Volume", "Transmit on Live Set load", "Send Channel Volume when this Live Set loads."),
    p("live_set.zone.transmit.pan", "Send Pan", "Transmit on Live Set load", "Send Pan when this Live Set loads."),
    p("live_set.zone.transmit_controllers.pitch_bend", "Forward Pitch Bend", "Forward live controllers", "Relay Pitch Bend to this Zone's channel."),
    p("live_set.zone.transmit_controllers.modulation", "Forward Modulation", "Forward live controllers", "Relay the Modulation wheel to this Zone's channel."),
    p("live_set.zone.transmit_controllers.foot_pedal_1", "Forward Foot Pedal 1", "Forward live controllers", "Relay foot pedal 1 to this Zone's channel."),
    p("live_set.zone.transmit_controllers.foot_pedal_2", "Forward Foot Pedal 2", "Forward live controllers", "Relay foot pedal 2 to this Zone's channel."),
    // ===== Live Set / Part =====
    p("live_set.part.current_category", "Category", "Voice", "Active sound category for this part (selects which category voice plays)."),
    pc("live_set.part.category_voices", "Category voices", "Voice", "Selected voice number per category for this part.", "voices"),
    p("live_set.part.note_shift", "Note shift", "Mix and tuning", "Transposes this part −24..+24 semitones."),
    pl("live_set.part.part_volume", "Volume", "Mix and tuning", "This part's volume (0–127)."),
    p("live_set.part.part_color", "Colour", "Mix and tuning", "Display colour for this part (0–11)."),
    p("live_set.part.part_switch", "Part on", "Mix and tuning", "Turns this part on or off."),
    p("live_set.part.part_selected", "Part selected", "Mix and tuning", "Whether this part is the one the panel controls edit."),
    p("live_set.part.effect_select", "Effect slot", "Insert effects", "Which insert-effect slot (1 or 2) the panel edits for this part."),
    p("live_set.part.pan", "Pan", "Mix and tuning", "Stereo pan for this part, −63..+63 (L63 .. C .. R63). Added in firmware v1.10."),
    p("live_set.part.mono_poly", "Mono/Poly", "Voice", "Plays this part monophonically or polyphonically."),
    p("live_set.part.mono_type", "Mono type", "Voice", "Mono behaviour: Normal, Fingered portamento, or Full-time portamento."),
    p("live_set.part.portamento_time", "Portamento time", "Voice", "Glide time between notes (0–127)."),
    p("live_set.part.portamento_mode", "Portamento mode", "Voice", "Whether portamento time is interpreted as a Rate or an absolute Time."),
    p("live_set.part.unison_switch", "Unison", "Unison", "Stacks detuned copies of the voice."),
    p("live_set.part.unison_type", "Unison type", "Unison", "Unison voicing: Multi Layer, Harmonics, or Sub Harmonics."),
    pl("live_set.part.unison_volume", "Unison volume", "Unison", "Level of the unison copies (0–127)."),
    pl("live_set.part.unison_detune", "Unison detune", "Unison", "Detuning spread of the unison copies (0–127)."),
    p("live_set.part.pitch_bend_range", "Pitch bend range", "Modulation", "Pitch-bend wheel range, −24..+24 semitones."),
    pl("live_set.part.pitch_modulation_depth", "Pitch mod depth", "Modulation", "Amount of pitch modulation (vibrato) from the mod wheel (0–127)."),
    pl("live_set.part.amplifier_modulation_depth", "Amp mod depth", "Modulation", "Amount of amplitude modulation (tremolo) from the mod wheel (0–127)."),
    pl("live_set.part.filter_modulation_depth", "Filter mod depth", "Modulation", "Amount of filter modulation (wah) from the mod wheel (0–127)."),
    p("live_set.part.modulation_speed", "Modulation speed", "Modulation", "LFO speed for the modulation, −64..+63."),
    p("live_set.part.receive_expression", "Receive Expression", "Receive / external", "This part responds to Expression (CC 11)."),
    p("live_set.part.receive_sustain", "Receive Sustain", "Receive / external", "This part responds to the Sustain pedal (CC 64)."),
    p("live_set.part.receive_sostenuto", "Receive Sostenuto", "Receive / external", "This part responds to Sostenuto (CC 66)."),
    p("live_set.part.receive_soft", "Receive Soft", "Receive / external", "This part responds to the Soft pedal (CC 67)."),
    p("live_set.part.external_keyboard", "External keyboard", "Receive / external", "How this part sounds from an external keyboard: Ext+Int, Ext only, or Off."),
    pl("live_set.part.touch_sensitivity_depth", "Touch depth", "Voice", "How strongly velocity affects this part (0–127)."),
    pl("live_set.part.touch_sensitivity_offset", "Touch offset", "Voice", "Baseline velocity added before the touch curve (0–127)."),
    p("live_set.part.drawbars", "Drawbars", "Organ", "The nine organ drawbar levels (16', 5⅓', 8', 4', 2⅔', 2', 1⅗', 1⅓', 1')."),
    p("live_set.part.percussion_switch", "Percussion", "Organ", "Organ percussion on/off."),
    p("live_set.part.percussion_type", "Percussion harmonic", "Organ", "Percussion pitch: 2nd or 3rd harmonic."),
    p("live_set.part.percussion_decay", "Percussion decay", "Organ", "Percussion decay: Slow or Fast."),
    p("live_set.part.percussion_volume", "Percussion volume", "Organ", "Percussion level: Normal or Soft."),
    p("live_set.part.vibrato_chorus_switch", "Vibrato/Chorus", "Organ", "Organ vibrato/chorus scanner on/off."),
    p("live_set.part.vibrato_chorus_type", "Vibrato/Chorus type", "Organ", "Scanner program: V1, C1, V2, C2, V3, or C3."),
    p("live_set.part.filter_switch", "Filter", "Filter and EG", "Enables the part filter."),
    pl("live_set.part.filter_cutoff", "Cutoff", "Filter and EG", "Filter cutoff frequency (0–127)."),
    pl("live_set.part.filter_resonance", "Resonance", "Filter and EG", "Filter resonance / emphasis (0–127)."),
    p("live_set.part.eg_switch", "EG", "Filter and EG", "Enables the amplitude envelope (attack/release) edits."),
    pl("live_set.part.eg_attack", "Attack", "Filter and EG", "Envelope attack time (0–127)."),
    pl("live_set.part.eg_release", "Release", "Filter and EG", "Envelope release time (0–127)."),
    p("live_set.part.drive_switch", "Drive", "Drive", "Enables the drive/amp stage."),
    p("live_set.part.drive_type", "Drive type", "Drive", "Drive model: Overdrive, Distortion, Rotary A, Rotary B, or Compressor."),
    pl("live_set.part.drive_depth", "Drive depth", "Drive", "Drive amount (0–127)."),
    p("live_set.part.effect_1_switch", "Effect 1", "Insert effects", "Insert effect 1 on/off."),
    pc("live_set.part.effect_1_type", "Effect 1 type", "Insert effects", "Algorithm for insert effect 1.", "part_effects"),
    pl("live_set.part.effect_1_depth", "Effect 1 depth", "Insert effects", "Depth/mix of insert effect 1 (0–127)."),
    pl("live_set.part.effect_1_rate", "Effect 1 rate", "Insert effects", "Rate of insert effect 1 (0–127)."),
    p("live_set.part.effect_2_switch", "Effect 2", "Insert effects", "Insert effect 2 on/off."),
    pc("live_set.part.effect_2_type", "Effect 2 type", "Insert effects", "Algorithm for insert effect 2.", "part_effects"),
    pl("live_set.part.effect_2_depth", "Effect 2 depth", "Insert effects", "Depth/mix of insert effect 2 (0–127)."),
    pl("live_set.part.effect_2_rate", "Effect 2 rate", "Insert effects", "Rate of insert effect 2 (0–127)."),
];

/// The CK parameter table as a [`Params`] handle (for the shared resolver / the
/// [`Device`](crate::Device) contract).
pub fn params() -> Params {
    Params(PARAMS)
}

/// Help text for a field path, or `None` if the path isn't in the catalog.
pub fn field_help(path: &str) -> Option<&'static str> {
    field_meta(path).map(|m| m.help)
}

/// Full metadata for a field path, or `None` if the path isn't in the catalog.
pub fn field_meta(path: &str) -> Option<&'static ParamMeta> {
    PARAMS.iter().find(|m| m.path == path)
}

/// All fields in a display group, in catalog order.
pub fn params_in_group(group: &str) -> Vec<&'static ParamMeta> {
    PARAMS.iter().filter(|m| m.group == group).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn paths_are_unique() {
        let mut seen = HashSet::new();
        for m in PARAMS {
            assert!(seen.insert(m.path), "duplicate path: {}", m.path);
        }
    }

    #[test]
    fn labels_groups_help_non_empty() {
        for m in PARAMS {
            assert!(!m.label.is_empty(), "empty label: {}", m.path);
            assert!(!m.group.is_empty(), "empty group: {}", m.path);
            assert!(!m.help.is_empty(), "empty help: {}", m.path);
        }
    }

    #[test]
    fn lookup_hits_and_misses() {
        assert!(field_meta("system.common.master_tune").is_some());
        assert_eq!(field_meta("system.common.does_not_exist"), None);
        assert_eq!(field_help("nope.nope"), None);
    }

    #[test]
    fn master_tune_help_text() {
        assert_eq!(
            field_help("system.common.master_tune"),
            Some("Master tuning of the whole instrument (414.72–466.78 Hz; default A=440).")
        );
    }

    #[test]
    fn level_flag_only_on_magnitudes() {
        use midi_access_core::meta::Level;
        assert_eq!(
            field_meta("live_set.part.filter_cutoff").unwrap().level,
            Level::Magnitude
        );
        // centred, not a magnitude
        assert_eq!(field_meta("live_set.part.pan").unwrap().level, Level::Plain);
        // signed
        assert_eq!(
            field_meta("system.master_eq.low_gain").unwrap().level,
            Level::Plain
        );
    }

    /// Cross-check: every editable field of each typed struct has a catalog
    /// entry. Field-name lists are kept in sync with the structs by hand.
    #[test]
    fn every_struct_field_has_meta() {
        let groups: &[(&str, &[&str])] = &[
            (
                "system.common",
                &[
                    "master_tune",
                    "keyboard_octave_shift",
                    "keyboard_transpose",
                    "controller_reset",
                    "local_control",
                    "tx_channel",
                    "rx_channel",
                    "midi_control",
                    "output_gain",
                    "touch_curve",
                    "fixed_velocity",
                    "tx_rx_bank_select",
                    "tx_rx_program_change",
                    "midi_in_out",
                    "usb_in_out",
                    "value_indication",
                    "controller_mode",
                    "lcd_switch",
                    "lcd_contrast",
                    "panel_lock_live_set",
                    "panel_lock_organ",
                    "panel_lock_filter_eg",
                    "panel_lock_drive_effect",
                    "panel_lock_delay_reverb",
                    "panel_lock_equalizer",
                    "power_on_page",
                    "power_on_sound",
                    "foot_pedal_1_type",
                    "foot_pedal_1_live_set",
                    "foot_pedal_2_type",
                    "foot_pedal_2_live_set",
                    "filter_eg_reset",
                    "effect_on_off_reset",
                    "usb_audio_volume",
                    "bluetooth_volume",
                    "ad_volume",
                    "usb_audio_loopback",
                    "speaker_mode",
                    "speaker_mute",
                ],
            ),
            (
                "system.master_eq",
                &[
                    "low_gain",
                    "low_freq",
                    "mid_gain",
                    "mid_freq",
                    "high_gain",
                    "high_freq",
                ],
            ),
            (
                "live_set.common",
                &[
                    "name",
                    "live_set_eq_mode",
                    "master_keyboard_mode",
                    "advanced_zone",
                    "tempo_x10",
                    "sound_transpose",
                    "layer_split_mode",
                    "split_point",
                    "split_point_a_b",
                    "split_point_b_c",
                    "mod_wheel_assign",
                    "foot_pedal_1_assign",
                    "foot_pedal_1_limit_low",
                    "foot_pedal_1_limit_high",
                    "foot_pedal_2_assign",
                    "foot_pedal_2_limit_low",
                    "foot_pedal_2_limit_high",
                    "delay_switch",
                    "delay_type",
                    "delay_depth",
                    "delay_time",
                    "delay_tempo_time",
                    "reverb_switch",
                    "reverb_type",
                    "reverb_depth",
                    "rotary_speed",
                    "rotary_stop",
                    "audio_trigger_switch",
                    "audio_trigger_volume",
                    "audio_trigger_key",
                    "audio_trigger_play_mode",
                    "ad_eq_low_freq",
                    "ad_eq_low_gain",
                    "ad_eq_mid_freq",
                    "ad_eq_mid_gain",
                    "ad_eq_high_freq",
                    "ad_eq_high_gain",
                    "ad_noise_gate_switch",
                    "ad_noise_gate_threshold",
                    "ad_effect_1_type",
                    "ad_effect_1_depth",
                    "ad_effect_1_rate",
                    "ad_effect_2_type",
                    "ad_effect_2_depth",
                    "ad_effect_2_rate",
                    "ad_volume",
                ],
            ),
            (
                "live_set.eq",
                &[
                    "low_gain",
                    "low_freq",
                    "mid_gain",
                    "mid_freq",
                    "high_gain",
                    "high_freq",
                ],
            ),
            (
                "live_set.rotary",
                &[
                    "a_balance",
                    "a_stereo_mono",
                    "a_horn_slow",
                    "a_rotor_slow",
                    "a_horn_fast",
                    "a_rotor_fast",
                    "a_horn_acceleration",
                    "a_rotor_acceleration",
                    "a_horn_deceleration",
                    "a_rotor_deceleration",
                    "b_balance",
                    "b_stereo_mono",
                    "b_horn_slow",
                    "b_rotor_slow",
                    "b_horn_fast",
                    "b_rotor_fast",
                    "b_horn_transition",
                    "b_rotor_transition",
                ],
            ),
            (
                "live_set.zone",
                &[
                    "zone_switch",
                    "transmit_channel",
                    "transpose_octave",
                    "transpose_semitone",
                    "note_limit_low",
                    "note_limit_high",
                    "midi_volume",
                    "midi_pan",
                    "midi_bank_msb",
                    "midi_bank_lsb",
                    "midi_program",
                ],
            ),
            (
                "live_set.part",
                &[
                    "current_category",
                    "category_voices",
                    "note_shift",
                    "part_volume",
                    "part_color",
                    "part_switch",
                    "part_selected",
                    "effect_select",
                    "pan",
                    "mono_poly",
                    "mono_type",
                    "portamento_time",
                    "portamento_mode",
                    "unison_switch",
                    "unison_type",
                    "unison_volume",
                    "unison_detune",
                    "pitch_bend_range",
                    "pitch_modulation_depth",
                    "amplifier_modulation_depth",
                    "filter_modulation_depth",
                    "modulation_speed",
                    "receive_expression",
                    "receive_sustain",
                    "receive_sostenuto",
                    "receive_soft",
                    "external_keyboard",
                    "touch_sensitivity_depth",
                    "touch_sensitivity_offset",
                    "drawbars",
                    "percussion_switch",
                    "percussion_type",
                    "percussion_decay",
                    "percussion_volume",
                    "vibrato_chorus_switch",
                    "vibrato_chorus_type",
                    "filter_switch",
                    "filter_cutoff",
                    "filter_resonance",
                    "eg_switch",
                    "eg_attack",
                    "eg_release",
                    "drive_switch",
                    "drive_type",
                    "drive_depth",
                    "effect_1_switch",
                    "effect_1_type",
                    "effect_1_depth",
                    "effect_1_rate",
                    "effect_2_switch",
                    "effect_2_type",
                    "effect_2_depth",
                    "effect_2_rate",
                ],
            ),
        ];
        for (area, fields) in groups {
            for field in *fields {
                let path = format!("{area}.{field}");
                assert!(field_meta(&path).is_some(), "missing meta for {path}");
            }
        }
        // Standalone path on LiveSet.
        assert!(field_meta("live_set.audio_trigger_path").is_some());
    }
}
