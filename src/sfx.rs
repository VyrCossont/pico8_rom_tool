use crate::rom;
use anyhow;
use packed_struct::prelude::*;
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
            .fold(true, |acc, x| acc && u8::from(x.volume) > 0)
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
#[derive(PackedStruct, Debug, Default)]
#[packed_struct(size_bytes = "2", bit_numbering = "lsb0")]
pub struct Note {
    #[packed_field(bits = "0..=5")]
    pub pitch: Integer<u8, packed_bits::Bits<6>>,
    #[packed_field(bits = "6..=8")]
    waveform: Integer<u8, packed_bits::Bits<3>>,
    #[packed_field(bits = "9..=11")]
    pub volume: Integer<u8, packed_bits::Bits<3>>,
    #[packed_field(bits = "12..=14", ty = "enum")]
    pub effect: Effect,
    #[packed_field(bits = "15")]
    sfx_instrument: bool,
}

impl Note {
    pub fn instrument(&self) -> Instrument {
        if self.sfx_instrument {
            Instrument::Sfx(self.waveform)
        } else {
            match u8::from(self.waveform) {
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
        self.sfx_instrument = match val {
            Instrument::Sfx(_) => true,
            _ => false,
        };
        self.waveform = match val {
            Instrument::Sfx(waveform) => waveform,
            Instrument::Triangle => Integer::from(0),
            Instrument::TiltedSaw => Integer::from(1),
            Instrument::Saw => Integer::from(2),
            Instrument::Square => Integer::from(3),
            Instrument::Pulse => Integer::from(4),
            Instrument::Organ => Integer::from(5),
            Instrument::Noise => Integer::from(6),
            Instrument::Phaser => Integer::from(7),
        };
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
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, Default)]
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
