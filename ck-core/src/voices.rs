//! Voice list — maps the Part's category voice numbers (0..=362) to names and
//! categories. Transcribed from the *Owner's Manual* "Voice List".
//!
//! The CK groups its 363 voices into 10 categories; a Part stores the selected
//! absolute voice number per category (see [`crate::part::Part::category_voices`]).
//! This module turns those numbers into human-readable names — the data the
//! editor needs to render voice pickers and the CLI uses in `show`.

/// The 10 top-level voice categories, in Part-index order (0..=9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Piano,
    EPiano,
    Organ,
    BrsWind,
    GtrBass,
    Strings,
    Pad,
    Lead,
    ChrPerc,
    Others,
}

impl Category {
    /// Part category index (0..=9).
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(i: u8) -> Option<Self> {
        use Category::*;
        Some(match i {
            0 => Piano,
            1 => EPiano,
            2 => Organ,
            3 => BrsWind,
            4 => GtrBass,
            5 => Strings,
            6 => Pad,
            7 => Lead,
            8 => ChrPerc,
            9 => Others,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        use Category::*;
        match self {
            Piano => "Piano",
            EPiano => "E.Piano",
            Organ => "Organ",
            BrsWind => "Brs/Wind",
            GtrBass => "Gtr/Bass",
            Strings => "Strings",
            Pad => "Pad",
            Lead => "Lead",
            ChrPerc => "Chr.Perc",
            Others => "Others",
        }
    }

    /// Inclusive absolute voice-number range owned by this category.
    pub fn voice_range(self) -> (u16, u16) {
        use Category::*;
        match self {
            Piano => (0, 12),
            EPiano => (13, 26),
            Organ => (27, 46),
            BrsWind => (47, 108),
            GtrBass => (109, 166),
            Strings => (167, 208),
            Pad => (209, 273),
            Lead => (274, 324),
            ChrPerc => (325, 350),
            Others => (351, 362),
        }
    }
}

/// Category owning the given absolute voice number, if any.
pub fn category_of(voice: u16) -> Option<Category> {
    (0..10).filter_map(Category::from_index).find(|c| {
        let (lo, hi) = c.voice_range();
        (lo..=hi).contains(&voice)
    })
}

/// Voice name for an absolute voice number (0..=362), or `None` if out of range.
pub fn voice_name(voice: u16) -> Option<&'static str> {
    VOICE_NAMES.get(voice as usize).copied()
}

/// All 363 voice names, indexed by absolute voice number.
pub static VOICE_NAMES: [&str; 363] = [
    // Piano (0..=12)
    "CFX Stereo",
    "CFX St Bright",
    "CFX St Warm",
    "CFX Mono",
    "CFX Mn Bright",
    "CFX Mn Warm",
    "S700",
    "Live CF3",
    "Digi Piano 1",
    "Digi Piano 2",
    "U1",
    "CP80 1",
    "CP80 2",
    // E.Piano (13..=26)
    "78Rd",
    "73Rd Studio",
    "Wr Warm",
    "Wr Bright",
    "Clavi B",
    "Clavi S",
    "Harpsi 1",
    "Harpsi 2",
    "DX Legend",
    "DX Woody",
    "DX FTine",
    "DX 7 II",
    "DX Mellow",
    "DX Crisp",
    // Organ (27..=46)
    "H",
    "V",
    "F",
    "A",
    "Y",
    "Pipe Organ 1",
    "Pipe Organ 2",
    "Concert Organ",
    "Grand Jeu",
    "FondsEtAnches",
    "Organo Pleno",
    "Diapason",
    "Claribel&Flut",
    "Soft Reeds",
    "Church Organ1",
    "Church Organ2",
    "Church Organ3",
    "Church Organ4",
    "Accordion",
    "Musette",
    // Brs/Wind (47..=108)
    "BrassSection1",
    "BrassSection2",
    "BrassSection3",
    "BrassSection4",
    "BrassSection5",
    "Stz Brass",
    "Forte Brass",
    "StorzandoFall",
    "High Brass",
    "Mellow Brass1",
    "Mellow Brass2",
    "Soft Brass",
    "Tp&Tb Section",
    "Trb. Section",
    "Horn Section",
    "Horn Strings",
    "Brass Strings",
    "Sweet Trumpet",
    "Trumpet",
    "Trombone",
    "French Horn",
    "Horn",
    "Sax Section 1",
    "Sax Section 2",
    "Sax Section 3",
    "Sweet Alto",
    "Alto Sax",
    "Tenor Sax 1",
    "Tenor Sax 2",
    "Soprano Sax",
    "Baritone Sax",
    "Oboe",
    "Bassoon",
    "Clarinet",
    "Flute 1",
    "Flute 2",
    "Alto Flute",
    "Tape Flute",
    "Recorder",
    "Pan Flute 1",
    "Pan Flute 2",
    "Bottle",
    "Shakuhachi",
    "Ocarina",
    "Harmonica 1",
    "Harmonica 2",
    "Bagpipe",
    "Synth Brass 1",
    "Synth Brass 2",
    "Synth Brass 3",
    "Synth Brass 4",
    "Jump Brass",
    "OB Brass 1",
    "OB Brass 2",
    "OB Brass 3",
    "OB Brass 4",
    "OB Brass 5",
    "SoftSynBrs1",
    "SoftSynBrs2",
    "Big Squish",
    "Analog Brass1",
    "Analog Brass2",
    // Gtr/Bass (109..=166)
    "Classic Gt",
    "Nylon Guitar1",
    "Nylon Gt Harm",
    "Nylon Guitar2",
    "Steel Gt 1",
    "Steel Gt 2",
    "Steel Gt 3",
    "12 Str Gt 1",
    "12 Str Gt 2",
    "Clean Gt 1",
    "Clean Gt 2",
    "Clean Gt 3",
    "60's Clean Gt",
    "Funk Guitar",
    "12 Str Clean",
    "Dist Guitar 1",
    "Dist Guitar 2",
    "Over The Top",
    "Crunch Guitar",
    "Crunch Oct",
    "Mute Dist",
    "Jazz Guitar",
    "Hawaiian Gt",
    "Acoustic Bass",
    "Upright Bass",
    "Finger Bass 1",
    "Finger Bass 2",
    "Finger Bass 3",
    "Finger Bass 4",
    "Pick Bass 1OM",
    "Pick Bass 1 M",
    "Pick Bass 1 O",
    "Pick Bass 2",
    "Slap Bass",
    "Fretless Ba 1",
    "Fretless Ba 2",
    "A.Bass + Cym",
    "E.Bass + Cym",
    "Synth Bass 1",
    "Synth Bass 2",
    "Synth Bass 3",
    "Synth Bass 4",
    "Synth Bass 5",
    "Big Bass",
    "101 Bass",
    "Competitor",
    "Perc Punch",
    "Trance Bass",
    "Dark Bass",
    "Click SynBass",
    "Acid Bass",
    "Square Bass",
    "Long Spit",
    "Fundamental",
    "One Voice",
    "Fat Sine",
    "Fat Sine Res",
    "Unison Bass",
    // Strings (167..=208)
    "Section Str 1",
    "Section Str 2",
    "Section Str 3",
    "Strings 1",
    "Strings 2",
    "Orchestra 1",
    "Orchestra 2",
    "Arco String",
    "Fast Strings",
    "Marcato Str",
    "Concert Str",
    "Legato Str",
    "Warm Strings",
    "Slow Str 1",
    "Slow Str 2",
    "Slow Str 3",
    "60's Strings",
    "70's Strings1",
    "70's Strings2",
    "SlwAtkTremolo",
    "Tremolo Str",
    "Velo Strings",
    "Quartet",
    "Tron Strings",
    "Tape Strings",
    "Flute Strings",
    "Sweet Violin",
    "Violin",
    "Cello",
    "Pizzicato 1",
    "Pizzicato 2",
    "Harp",
    "Syn Strings 1",
    "Syn Strings 2",
    "Syn Strings 3",
    "Analog Str",
    "Lite Strings1",
    "Lite Strings2",
    "JP Strings",
    "Pop Syn Str",
    "Unison Str",
    "Oct Syn Str",
    // Pad (209..=273)
    "Bell Pad 1",
    "Bell Pad 2",
    "BrightPadBell",
    "Sharp Teeth",
    "Ring Pad",
    "Anlg Rez Pad",
    "LFO Pad",
    "Chill Scap",
    "Strings Pad",
    "Back Pad",
    "Planet",
    "Atmosphere",
    "Click Pad",
    "Pad 80",
    "Poly Pad",
    "Glass Harp",
    "Digi Stuff",
    "New Age Pad",
    "Darklight",
    "Neo Crystal",
    "Vapor",
    "Soft Pad 1",
    "Soft Pad 2",
    "VP Soft",
    "Glass Pad",
    "Sine Pad",
    "Échoes",
    "Ambient Pad",
    "Pan Pad",
    "Sci-Fi",
    "Big Pad",
    "Goblins",
    "Sweep Pad 1",
    "Sweep Pad 2",
    "Nowhere",
    "Goblins Synth",
    "Celestial",
    "Converge",
    "Creation",
    "Ancestral",
    "Soundtrack",
    "Echo Pad",
    "Rain",
    "Analog Pad",
    "Dark Light",
    "Digi Pad",
    "Noble Pad",
    "Pop Pad",
    "Fat Saw",
    "Angel Pad",
    "Choir 1",
    "Choir 2",
    "Choir 3",
    "Air Choir",
    "Choir Aah",
    "Voice Oohs",
    "Slow Vox",
    "Slow Choir",
    "Itopia",
    "Mystic Pad",
    "Twist",
    "Da Pad",
    "Dark Star",
    "Mind Bell",
    "ZEN",
    // Lead (274..=324)
    "Dancy Hook",
    "Faaat Dance",
    "Techno Brass",
    "After 1984",
    "Analog Lead 1",
    "Analog Lead 2",
    "Analog Lead 3",
    "Analog Lead 4",
    "Saw Lead 1",
    "Saw Lead 2",
    "Saw Lead 3",
    "Wire Lead",
    "Classic Mini",
    "Big Lead 1",
    "Big Lead 2",
    "Early Lead",
    "Troy",
    "Sync Saw Lead",
    "Punch Lead",
    "Soft RnB",
    "Popcorn",
    "Synth Trumpet",
    "Dynmic Mini",
    "Crying",
    "Funky Mini",
    "Funky Poly",
    "Mini Three",
    "Nu Mini",
    "Sky Walk",
    "Mini Soft",
    "Mini Lead",
    "Inda Night",
    "Sine Lead",
    "Tiny Lead",
    "Synth Whistle",
    "Raplead",
    "Funk Lead 1",
    "Funk Lead 2",
    "Rezz Punch",
    "Square Lead 1",
    "Square Lead 2",
    "Square Lead 3",
    "Soft Square",
    "5th Lead",
    "Digital Lead",
    "Voice Lead",
    "Wind Lead",
    "Calliope Ld 1",
    "Calliope Ld 2",
    "Orchestra Hit",
    "Impact",
    // Chr.Perc (325..=350)
    "Marimba 1",
    "Marimba 2",
    "Xylophone 1",
    "Xylophone 2",
    "Balimba",
    "Vib ST",
    "Vibraphone",
    "Hard Vibes",
    "Glocken 1",
    "Glocken 2",
    "Music Box",
    "Soft Crystal",
    "Tinkle Bell",
    "Tubular Bell",
    "Carillon",
    "Digi Bell 1",
    "Digi Bell 2",
    "Digi Bell 3",
    "Nice Bell",
    "Stack Bell",
    "Bell Harp",
    "Harp Vox",
    "Round Glock",
    "Air Bells",
    "Star Dust",
    "Heaven Bell",
    // Others (351..=362)
    "Kalimba",
    "Kanoon",
    "Shamisen",
    "Sitar 1",
    "Sitar 2",
    "Banjo",
    "Mandolin",
    "Dulcimer",
    "Koto",
    "Timpani",
    "Steel Drums",
    "Agogo",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_363_entries() {
        assert_eq!(VOICE_NAMES.len(), 363);
    }

    #[test]
    fn category_ranges_cover_table_contiguously() {
        // The 10 ranges must tile 0..=362 with no gaps or overlaps.
        let mut next = 0u16;
        for i in 0..10 {
            let (lo, hi) = Category::from_index(i).unwrap().voice_range();
            assert_eq!(lo, next, "gap/overlap at category {i}");
            next = hi + 1;
        }
        assert_eq!(next, 363);
    }

    #[test]
    fn known_voices_resolve() {
        assert_eq!(voice_name(0), Some("CFX Stereo"));
        assert_eq!(voice_name(6), Some("S700"));
        assert_eq!(voice_name(27), Some("H"));
        assert_eq!(voice_name(362), Some("Agogo"));
        assert_eq!(voice_name(363), None);
    }

    #[test]
    fn category_lookup() {
        assert_eq!(category_of(0), Some(Category::Piano));
        assert_eq!(category_of(108), Some(Category::BrsWind));
        assert_eq!(category_of(109), Some(Category::GtrBass));
        assert_eq!(category_of(362), Some(Category::Others));
        assert_eq!(category_of(999), None);
    }
}
