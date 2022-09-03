use crate::music::Pattern;
use crate::music::Section as MusicSection;
use crate::rom;
use crate::sfx::Section as SfxSection;
use crate::sfx::{Effect, Instrument, Sfx};
use anyhow;
use packed_struct::prelude::*;
use std::path::Path;

pub fn translate(path: &Path) -> anyhow::Result<()> {
    let section = rom::read_section::<MusicSfx>(path, 0x3100)?;

    let mut wasm4sfxes = Vec::with_capacity(section.sfx.sfxes.len());
    for (i, sfx) in section.sfx.sfxes.iter().enumerate() {
        match map_sfx(sfx) {
            Ok(wasm4sfx) => wasm4sfxes.push(wasm4sfx),
            Err(e) => eprintln!("Skipping SFX {}: {}", i, e),
        }
    }

    let mut wasm4patterns = Vec::with_capacity(section.music.patterns.len());
    for (i, pattern) in section.music.patterns.iter().enumerate() {
        match map_music(pattern) {
            Ok(wasm4pattern) => wasm4patterns.push(wasm4pattern),
            Err(e) => eprintln!("Skipping pattern {}: {}", i, e),
        }
    }

    // TODO: figure out much better way to emit this code
    println!("//region SFX and music data");
    println!();

    println!("const SFX_DATA: &[Sfx] = &[");
    for wasm4sfx in wasm4sfxes {
        println!("    Sfx{{");
        println!("        frames_per_tone: {},", wasm4sfx.frames_per_tone);
        if let Some(loop_restart) = wasm4sfx.loop_restart {
            println!("        loop_restart: Some({}),", loop_restart);
        } else {
            println!("        loop_restart: None,");
        }
        println!("        tones: &[");
        for tone in wasm4sfx.tones {
            println!("            Tone{{");
            println!("                frequency: {},", tone.frequency);
            println!("                duration: {},", tone.duration);
            println!("                volume: {},", tone.volume);
            println!("                flags: {},", tone.flags);
            println!("            }},");
        }
        println!("        ],");
        println!("    }},");
    }
    println!("];");
    println!();

    println!("const MUSIC_DATA: &[Pattern] = &[");
    for wasm4pattern in wasm4patterns {
        println!("    Pattern{{");
        println!("        loop_start: {},", wasm4pattern.loop_start);
        println!("        loop_back: {},", wasm4pattern.loop_back);
        println!("        stop_at_end: {},", wasm4pattern.stop_at_end);
        println!("        sfxes: &[");
        for sfx_id in wasm4pattern.sfx_ids {
            println!("            &SFX_DATA[{}],", sfx_id);
        }
        println!("        ],");
        println!("    }},");
    }
    println!("];");
    println!();

    println!("//endregion SFX and music data");
    Ok(())
}

fn map_music(pattern: &Pattern) -> anyhow::Result<Wasm4Pattern> {
    // TODO: should we skip empty patterns? Does PICO-8 actually play them?
    if !pattern.enabled() {
        anyhow::bail!("No channels in this pattern");
    }

    // TODO: check for inter-channel instrument conflicts

    Ok(Wasm4Pattern {
        loop_start: pattern.loop_start(),
        loop_back: pattern.loop_back(),
        stop_at_end: pattern.stop_at_end(),
        sfx_ids: pattern
            .channels
            .iter()
            .map(|c| u8::from(c.sfx_id) as usize)
            .collect(),
    })
}

fn map_sfx(sfx: &Sfx) -> anyhow::Result<Wasm4Sfx> {
    // TODO: we can't actually skip every silent SFX,
    //  as they might be used by music as spacers.
    //  But for now, skip them so we don't have empty SFXes everywhere.
    if !sfx.enabled() {
        anyhow::bail!("No notes in this SFX");
    }

    // Check preconditions for entire SFX.
    let frames_per_tone = match sfx.speed {
        0 => anyhow::bail!("PICO-8 speed 0 isn't representable in the PICO-8 tracker and you probably shouldn't use it"),
        s if s % 2 != 0 => anyhow::bail!("Odd PICO-8 speeds map to non-integer numbers of WASM-4 frames, and cannot be represented"),
        s => s / 2,
    };
    if sfx.switches.buzz {
        anyhow::bail!("Unsupported SFX filter: buzz");
    }
    if sfx.switches.noiz {
        anyhow::bail!("Unsupported SFX filter: noiz");
    }
    if sfx.switches.detune() != 0 {
        anyhow::bail!("Unsupported SFX filter: detune");
    }
    if sfx.switches.reverb() != 0 {
        anyhow::bail!("Unsupported SFX filter: reverb");
    }
    if sfx.switches.dampen() != 0 {
        anyhow::bail!("Unsupported SFX filter: dampen");
    }

    // Get SFX size and optional loop restart point.
    let (loop_restart, size) = match (sfx.loop_start, sfx.loop_end) {
        (0, 0) => (None, sfx.notes.len()),
        (size, 0) => (None, size as usize),
        (loop_restart, size) => (Some(loop_restart as usize), size as usize),
    };

    // Check preconditions for representable notes.
    for note in sfx.notes[..size].iter() {
        match note.instrument() {
            Instrument::Triangle => (),
            Instrument::Pulse => (),
            Instrument::Square => (),
            Instrument::Noise => (),
            instrument => anyhow::bail!("Unsupported instrument: {:#?}", instrument),
        }
        if note.effect() != Effect::None {
            anyhow::bail!("Unsupported effect: {:#?}", note.effect());
        }
    }

    let tones = sfx.notes[..size]
        .iter()
        .map(|note| Wasm4Tone {
            // TODO: emulate drop effect using frequency sweep?
            frequency: note.pitch().frequency(),
            // TODO: emulate other filters/effects using ADSR params?
            duration: frames_per_tone as u32,
            volume: (u8::from(note.volume()) as u32) * 100 / 7,
            // TODO: specify channel to use for pulse/square tones
            flags: match note.instrument() {
                Instrument::Triangle => 0b10,
                // pulse channel 1, default duty cycle
                Instrument::Pulse => 0b00,
                // pulse channel 2, 50% duty cycle
                Instrument::Square => 0b10_01,
                Instrument::Noise => 0b11,
                // We checked for this above.
                instrument => panic!("Unsupported instrument: {:#?}", instrument),
            },
        })
        .collect::<Vec<_>>();
    Ok(Wasm4Sfx {
        frames_per_tone,
        loop_restart,
        tones,
    })
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
struct Wasm4Sfx {
    frames_per_tone: u8,
    loop_restart: Option<usize>,
    tones: Vec<Wasm4Tone>,
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

#[derive(Debug)]
struct Wasm4Pattern {
    loop_start: bool,
    loop_back: bool,
    stop_at_end: bool,
    sfx_ids: Vec<usize>,
}
