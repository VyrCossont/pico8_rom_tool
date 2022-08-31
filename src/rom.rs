use anyhow;
use packed_struct::types::bits::ByteArray;
use packed_struct::PackedStruct;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn read_section<T: PackedStruct>(path: &Path, offset: u64) -> anyhow::Result<T> {
    path.file_name()
        .map(|x| x.to_string_lossy())
        .map(|x| x.ends_with(".p8.rom"))
        .filter(|x| *x)
        .ok_or(anyhow::anyhow!(
            "Not a supported PICO-8 ROM format: {}",
            path.to_string_lossy()
        ))?;
    let mut rom = File::open(path)?;
    rom.seek(SeekFrom::Start(offset))?;
    let mut buf = T::ByteArray::new(0);
    rom.read_exact(buf.as_mut_bytes_slice())?;
    T::unpack(&buf).map_err(|e| anyhow::anyhow!(e))
}

pub fn dump_section<T: PackedStruct + Debug>(path: &Path, offset: u64) -> anyhow::Result<()> {
    let section = read_section::<T>(path, offset)?;
    println!("{:#?}", section);
    Ok(())
}
