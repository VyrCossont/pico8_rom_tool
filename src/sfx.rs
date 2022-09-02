use crate::rom;
use anyhow;
use packed_struct::prelude::*;
use packed_struct::PrimitiveEnum;
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use std::path::Path;

pub fn dump(path: &Path) -> anyhow::Result<()> {
    rom::dump_section::<Section>(path, 0x3200)
}

/// See https://pico-8.fandom.com/wiki/Memory#Sound_effects
#[derive(PackedStruct, Debug)]
pub struct Section {
    #[packed_field(element_size_bytes = "68")]
    pub sfxes: [Sfx; 64],
}

/// `Default` is only defined for arrays up to size 32.
impl Default for Section {
    fn default() -> Self {
        Section {
            sfxes: [
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
                Sfx::default(),
            ],
        }
    }
}

#[derive(PackedStruct, Debug, Default)]
pub struct Sfx {
    #[packed_field(element_size_bytes = "2")]
    pub notes: [Note; 32],
    #[packed_field(size_bytes = "1")]
    pub switches: Switches,
    pub speed: u8,
    pub loop_start: u8,
    pub loop_end: u8,
}

impl Sfx {
    pub fn enabled(&self) -> bool {
        self.notes
            .iter()
            .fold(false, |acc, x| acc || u8::from(x.volume()) > 0)
    }
}

#[derive(PackedStruct, Debug, Default)]
#[packed_struct(size_bytes = "1", bit_numbering = "lsb0")]
pub struct Switches {
    #[packed_field(bits = "0")]
    pub tracker_mode: bool,
    #[packed_field(bits = "1")]
    pub noiz: bool,
    #[packed_field(bits = "2")]
    pub buzz: bool,
    #[packed_field(bits = "3..=7")]
    other_filters: Integer<u8, packed_bits::Bits<5>>,
}

impl Switches {
    pub fn detune(&self) -> u8 {
        u8::from(self.other_filters) % 3
    }

    pub fn set_detune(&mut self, val: u8) {
        if val > 2 {
            panic!("Detune must be 0–2; {} is out of range", val);
        }
        self.other_filters = Integer::from(val + self.reverb() * 3 + self.dampen() * 9);
    }

    pub fn reverb(&self) -> u8 {
        (u8::from(self.other_filters) / 3) % 3
    }

    pub fn set_reverb(&mut self, val: u8) {
        if val > 2 {
            panic!("Reverb must be 0–2; {} is out of range", val);
        }
        self.other_filters = Integer::from(self.detune() + val * 3 + self.dampen() * 9);
    }

    pub fn dampen(&self) -> u8 {
        (u8::from(self.other_filters) / 9) % 3
    }

    pub fn set_dampen(&mut self, val: u8) {
        if val > 2 {
            panic!("Dampen must be 0–2; {} is out of range", val);
        }
        self.other_filters = Integer::from(self.detune() + self.reverb() * 3 + val * 9);
    }
}

/// See https://www.lexaloffle.com/dl/docs/pico-8_manual.html#SFX_Editor
/// All the accessors are workarounds for https://github.com/hashmismatch/packed_struct.rs/issues/92
#[derive(PackedStruct, Debug, Default)]
#[packed_struct(size_bytes = "2", endian = "lsb")]
pub struct Note {
    packed: u16,
}

impl Note {
    fn mask(bits: &RangeInclusive<u8>) -> u16 {
        ((1u16 << bits.len()) - 1u16) << bits.start()
    }

    fn shift(bits: &RangeInclusive<u8>) -> u8 {
        *bits.start()
    }

    fn read(&self, bits: &RangeInclusive<u8>) -> u8 {
        ((self.packed & Self::mask(bits)) >> Self::shift(bits)) as u8
    }

    fn write(&mut self, bits: &RangeInclusive<u8>, val: u8) {
        self.packed |= Self::mask(bits) & ((val as u16) << Self::shift(bits))
    }

    const PITCH_BITS: RangeInclusive<u8> = 0..=5;
    const WAVEFORM_BITS: RangeInclusive<u8> = 6..=8;
    const VOLUME_BITS: RangeInclusive<u8> = 9..=11;
    const EFFECT_BITS: RangeInclusive<u8> = 12..=14;
    const SFX_INSTRUMENT_BITS: RangeInclusive<u8> = 15..=15;

    pub fn pitch(&self) -> Pitch {
        Pitch::from(self.read(&Self::PITCH_BITS))
    }

    pub fn set_pitch(&mut self, val: Pitch) {
        self.write(&Self::PITCH_BITS, u8::from(val));
    }

    pub fn volume(&self) -> Integer<u8, packed_bits::Bits<3>> {
        Integer::from(self.read(&Self::VOLUME_BITS))
    }

    pub fn set_volume(&mut self, val: Integer<u8, packed_bits::Bits<3>>) {
        self.write(&Self::VOLUME_BITS, u8::from(val));
    }

    pub fn effect(&self) -> Effect {
        Effect::from_primitive(self.read(&Self::EFFECT_BITS)).expect("Impossible effect number")
    }

    pub fn set_effect(&mut self, val: Effect) {
        self.write(&Self::EFFECT_BITS, val.to_primitive());
    }

    pub fn instrument(&self) -> Instrument {
        let sfx_instrument = self.read(&Self::SFX_INSTRUMENT_BITS) == 1;
        let waveform = self.read(&Self::WAVEFORM_BITS);
        if sfx_instrument {
            Instrument::Sfx(Integer::from(waveform))
        } else {
            match waveform {
                0 => Instrument::Triangle,
                1 => Instrument::TiltedSaw,
                2 => Instrument::Saw,
                3 => Instrument::Square,
                4 => Instrument::Pulse,
                5 => Instrument::Organ,
                6 => Instrument::Noise,
                7 => Instrument::Phaser,
                waveform => panic!("Impossible waveform number: {}", waveform),
            }
        }
    }

    pub fn set_instrument(&mut self, val: Instrument) {
        let sfx_instrument = match val {
            Instrument::Sfx(_) => 1,
            _ => 0,
        };
        self.write(&Self::SFX_INSTRUMENT_BITS, sfx_instrument);
        let waveform = match val {
            Instrument::Sfx(waveform) => u8::from(waveform),
            Instrument::Triangle => 0,
            Instrument::TiltedSaw => 1,
            Instrument::Saw => 2,
            Instrument::Square => 3,
            Instrument::Pulse => 4,
            Instrument::Organ => 5,
            Instrument::Noise => 6,
            Instrument::Phaser => 7,
        };
        self.write(&Self::WAVEFORM_BITS, waveform);
    }
}

#[derive(Debug, Default)]
pub enum Instrument {
    #[default]
    Triangle,
    TiltedSaw,
    Saw,
    Square,
    Pulse,
    Organ,
    Noise,
    Phaser,
    Sfx(Integer<u8, packed_bits::Bits<3>>),
}

/// See https://www.lexaloffle.com/dl/docs/pico-8_manual.html#Effects
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, Default, PartialEq)]
pub enum Effect {
    #[default]
    None = 0,
    Slide = 1,
    Vibrato = 2,
    Drop = 3,
    FadeIn = 4,
    FadeOut = 5,
    ArpeggioFast = 6,
    ArpeggioSlow = 7,
}

pub struct Pitch(u8);

/// MIDI note number and frequency assume MIDI Tuning Standard (12-tone equal temperament, middle C is C4).
impl Pitch {
    /// ⚠️ PICO-8 octave 2 is MTS octave 4. This returns the note's PICO-8 octave.
    pub fn octave(&self) -> u8 {
        self.0 / 12
    }

    const NAMES: [&'static str; 12] = [
        "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
    ];

    pub fn name(&self) -> &str {
        Self::NAMES[(self.0 % 12) as usize]
    }

    pub fn midi_note_number(&self) -> u8 {
        self.0 + 36
    }

    pub fn frequency(&self) -> u32 {
        (440.0 * 2.0f64.powf((self.midi_note_number() as f64 - 69.0) / 12.0)) as u32
    }
}

impl From<u8> for Pitch {
    fn from(val: u8) -> Self {
        if val > 63 {
            panic!("PICO-8 cannot represent notes above D♯5");
        }
        Pitch(val)
    }
}

impl From<Pitch> for u8 {
    fn from(val: Pitch) -> Self {
        val.0
    }
}

impl Display for Pitch {
    /// Shows note with PICO-8 octave.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} ({} Hz)",
            self.name(),
            self.octave(),
            self.frequency()
        )
    }
}
