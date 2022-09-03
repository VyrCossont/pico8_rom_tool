use crate::rom;
use anyhow;
use packed_struct::prelude::*;
use std::path::Path;

pub fn dump(path: &Path) -> anyhow::Result<()> {
    rom::dump_section::<Section>(path, 0x3100)
}

/// See https://pico-8.fandom.com/wiki/Memory#Music
#[derive(PackedStruct, Debug)]
pub struct Section {
    #[packed_field(element_size_bytes = "4")]
    pub patterns: [Pattern; 64],
}

/// `Default` is only defined for arrays up to size 32.
impl Default for Section {
    fn default() -> Self {
        Section {
            patterns: [
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
                Pattern::default(),
            ],
        }
    }
}

#[derive(PackedStruct, Debug, Default)]
#[packed_struct()]
pub struct Pattern {
    #[packed_field(element_size_bytes = "1")]
    pub channels: [Channel; 4],
}

impl Pattern {
    pub fn loop_start(&self) -> bool {
        self.channels[0].pattern_flag
    }

    pub fn set_begin_loop(&mut self, val: bool) {
        self.channels[0].pattern_flag = val
    }

    pub fn loop_back(&self) -> bool {
        self.channels[1].pattern_flag
    }

    pub fn set_end_loop(&mut self, val: bool) {
        self.channels[1].pattern_flag = val
    }

    pub fn stop_at_end(&self) -> bool {
        self.channels[2].pattern_flag
    }

    pub fn set_stop_at_end(&mut self, val: bool) {
        self.channels[2].pattern_flag = val
    }

    pub fn enabled(&self) -> bool {
        self.channels
            .iter()
            .fold(false, |acc, x| acc || x.enabled())
    }
}

#[derive(PackedStruct, Debug, Default)]
#[packed_struct(size_bytes = "1", bit_numbering = "lsb0")]
pub struct Channel {
    #[packed_field(bits = "0..=5")]
    pub sfx_id: Integer<u8, packed_bits::Bits<6>>,
    #[packed_field(bits = "6")]
    disabled: bool,
    #[packed_field(bits = "7")]
    pattern_flag: bool,
}

impl Channel {
    pub fn enabled(&self) -> bool {
        !self.disabled
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.disabled = !val
    }
}
