use crate::music::Section as MusicSection;
use crate::rom;
use crate::sfx::Instrument;
use crate::sfx::Section as SfxSection;
use anyhow;
use packed_struct::prelude::*;
use std::path::Path;

pub fn translate(path: &Path) -> anyhow::Result<()> {
    let section = rom::read_section::<MusicSfx>(path, 0x3100)?;
    let sfx = &section.sfx.sfxes[0];
    let frames_per_note = match sfx.speed {
        0 => return Err(anyhow::anyhow!("PICO-8 speed 0 isn't representable in the PICO-8 tracker and you probably shouldn't use it")),
        s if s % 2 != 0 => return Err(anyhow::anyhow!("Odd PICO-8 speeds map to non-integer numbers of WASM-4 frames, and cannot be represented")),
        s => s / 2,
    };
    let len: usize = match (sfx.loop_start, sfx.loop_end) {
        (0, 0) => 32,
        (x, 0) => x as usize,
        _ => todo!("Looping sfx are not supported yet"),
    };
    let tones = sfx.notes[..len]
        .iter()
        .map(|note| Wasm4Tone {
            frequency: note.pitch().frequency(),
            duration: frames_per_note as u32,
            volume: (u8::from(note.volume()) as u32) * 100 / 7,
            flags: match note.instrument() {
                Instrument::Triangle => 0b10,
                // pulse channel 1, default duty cycle
                Instrument::Pulse => 0b00,
                // pulse channel 2, 50% duty cycle
                Instrument::Square => 0b10_01,
                Instrument::Noise => 0b11,
                instrument => panic!("Unsupported instrument: {:#?}", instrument),
            },
        })
        .collect::<Vec<_>>();
    let wasm4sfx = Wasm4Sfx {
        frames_per_tone: frames_per_note,
        frame_counter: 0,
        tone_counter: 0,
        tones: tones.as_slice(),
    };
    println!("{:#?}", wasm4sfx);
    Ok(())
}

// Section type aliases are necessary because PackedStruct can't handle qualified field types.
/// The music and sfx sections are contiguous in a PICO-8 ROM.
#[derive(PackedStruct, Debug)]
pub struct MusicSfx {
    #[packed_field(size_bytes = "256")]
    music: MusicSection,
    #[packed_field(size_bytes = "4352")]
    sfx: SfxSection,
}

#[derive(Debug)]
struct Wasm4Sfx<'a> {
    frames_per_tone: u8,
    frame_counter: u8,
    tone_counter: u8,
    tones: &'a [Wasm4Tone],
}

/// Parameters for a WASM-4 `tone(…)` call.
/// See https://wasm4.org/docs/reference/functions/#tone-frequency-duration-volume-flags
#[derive(Debug)]
struct Wasm4Tone {
    frequency: u32,
    duration: u32,
    volume: u32,
    flags: u32,
}
