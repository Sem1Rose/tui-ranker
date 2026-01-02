use crate::bitfield::BitField;
use csv::{ReaderBuilder, WriterBuilder};
use std::{fs::File, io::prelude::*, path::Path, str::FromStr};

pub fn get_cached_items<T: FromStr>(dir: &Path) -> anyhow::Result<Vec<(T, f32)>> {
    let mut items = vec![];

    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(dir.join(".items"))?;
    for result in reader.records() {
        let record = result?;

        // for some reason, not using `ok()` throws a stupid error saying that <T as FromStr>::Err does not implement `std::fmt::Debug`??!!!
        items.push((
            T::from_str(&record[0].to_string()).ok().unwrap(),
            record[1].parse().unwrap(),
        ));
    }

    Ok(items)
}
pub fn get_cached_results(dir: &Path) -> anyhow::Result<Vec<BitField>> {
    let mut file = File::open(dir.join(".results"))?;
    let mut results = vec![];

    let mut buf = [0u8; 2];
    let mut n = file.read(&mut buf)?;
    let bitmask_size = ((buf[0] as u16) << 8) | buf[1] as u16;
    if n == 0 || bitmask_size == 0 {
        return Ok(vec![]);
    }

    let mut buf = vec![0; bitmask_size as usize];
    n = file.read(&mut buf)?;
    while n > 0 {
        results.push(buf.into());

        buf = vec![0; bitmask_size as usize];
        n = file.read(&mut buf)?;
    }

    Ok(results)
}
pub fn get_cached_bitmasks(dir: &Path) -> anyhow::Result<Vec<BitField>> {
    let mut file = File::open(dir.join(".bitmasks"))?;
    let mut bitmasks = vec![];

    let mut buf = [0u8; 2];
    let mut n = file.read(&mut buf)?;
    let bitmask_size = ((buf[0] as u16) << 8) | buf[1] as u16;
    if n == 0 || bitmask_size == 0 {
        return Ok(vec![]);
    }

    let mut buf = vec![0; bitmask_size as usize];
    n = file.read(&mut buf)?;
    while n > 0 {
        bitmasks.push(buf.into());

        buf = vec![0; bitmask_size as usize];
        n = file.read(&mut buf)?;
    }

    Ok(bitmasks)
}

pub fn cache_items<T: ToString + Clone>(dir: &Path, items: &[(T, f32)]) -> anyhow::Result<()> {
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path(dir.join(".items"))?;

    for i in items {
        writer.write_field(i.0.to_string())?;
        writer.write_field(i.1.to_string())?;
        writer.write_record(None::<&[u8]>)?;
    }

    Ok(())
}
pub fn cache_results(dir: &Path, results: &[BitField]) -> anyhow::Result<()> {
    let mut file: File = File::create(dir.join(".results"))?;

    if results.is_empty() {
        _ = file.write(&[0])?;
        return Ok(());
    }

    let bitmask_size = (results.len() as u16 >> 3)
        + if results.len() as u16 & 0b111 != 0 {
            1
        } else {
            0
        };
    _ = file.write(&[(bitmask_size >> 8) as u8, bitmask_size as u8])?;

    for mut bitmask in results.iter().cloned() {
        bitmask.fit_to_bytes(bitmask_size as u16);

        _ = file.write_all(bitmask.ref_vec());
    }

    Ok(())
}
pub fn cache_bitmasks(dir: &Path, bitmasks: &[BitField]) -> anyhow::Result<()> {
    let mut file: File = File::create(dir.join(".bitmasks"))?;

    if bitmasks.is_empty() {
        _ = file.write(&[0])?;
        return Ok(());
    }

    let bitmask_size = (bitmasks.len() as u16 >> 3)
        + if bitmasks.len() as u16 & 0b111 != 0 {
            1
        } else {
            0
        };
    _ = file.write(&[(bitmask_size >> 8) as u8, bitmask_size as u8])?;

    for mut bitmask in bitmasks.iter().cloned() {
        bitmask.fit_to_bytes(bitmask_size as u16);

        _ = file.write_all(bitmask.ref_vec());
    }

    Ok(())
}

#[cfg(test)]
mod read_write_test {
    // use super::*;

    // #[test]
    // fn test_initialize() -> anyhow::Result<()> {
    //     initialize_files()?;

    //     assert!(Path::new(".bitmasks").exists());
    //     assert!(Path::new(".items").exists());
    //     assert!(Path::new(".results").exists());

    //     Ok(())
    // }

    // #[test]
    // fn test_items() -> anyhow::Result<()> {
    //     let items = vec![
    //         ("o".to_string(), 2.7),
    //         ("r".into(), 545.44),
    //         ("g".into(), 748.0),
    //     ];

    //     cache_items(&items)?;
    //     assert_eq!(items, get_cached_items()?);

    //     Ok(())
    // }

    // #[test]
    // fn test_results() -> anyhow::Result<()> {
    //     let items = vec![
    //         vec![].into(),
    //         vec![0x04, 2, 0].into(),
    //         vec![0x4, 10, 100].into(),
    //         vec![1].into(),
    //         vec![0x04, 1].into(),
    //     ];

    //     cache_results(&items)?;
    //     assert_eq!(
    //         vec![
    //             BitMask::from(vec![0, 0, 0, 0, 0]),
    //             vec![0x04, 2, 0, 0, 0].into(),
    //             vec![0x04, 10, 100, 0, 0].into(),
    //             vec![1, 0, 0, 0, 0].into(),
    //             vec![0x04, 1, 0, 0, 0].into(),
    //         ],
    //         get_cached_results()?
    //     );

    //     Ok(())
    // }

    // #[test]
    // fn test_bitmasks() -> anyhow::Result<()> {
    //     let items = vec![];

    //     cache_bitmasks(&items)?;
    //     assert_eq!(items, get_cached_bitmasks()?);

    //     let items = vec![
    //         vec![].into(),
    //         vec![0x04, 2, 0].into(),
    //         vec![0x4, 10, 100].into(),
    //         vec![1].into(),
    //         vec![0x04, 1].into(),
    //         vec![0x40].into(),
    //         vec![84].into(),
    //         vec![0xf4, 0xa, 0xff].into(),
    //         vec![0xa4].into(),
    //     ];

    //     cache_bitmasks(&items)?;
    //     assert_eq!(
    //         vec![
    //             BitMask::from(vec![0, 0]),
    //             vec![0x04, 2].into(),
    //             vec![0x04, 10].into(),
    //             vec![1, 0].into(),
    //             vec![0x04, 1].into(),
    //             vec![0x40, 0].into(),
    //             vec![84, 0].into(),
    //             vec![0xf4, 10].into(),
    //             vec![0xa4, 0].into()
    //         ],
    //         get_cached_bitmasks()?
    //     );

    //     Ok(())
    // }
}
