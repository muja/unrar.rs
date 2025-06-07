use unrar::Archive;

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn test_struct_size() {
    use unrar_sys::HeaderDataEx;
    assert_eq!(std::mem::size_of::<HeaderDataEx>(), 14340);
}

#[cfg(feature="checksum")]
#[test]
fn test_blake2_checksum() {
    let archive = Archive::new("./data/blake2.rar").open_for_listing().unwrap();
    for entry in archive.into_iter() {
        let entry = entry.unwrap();
        assert!(entry.is_file());
        let file_hash_is_blake2 = match entry.file_crc {
            unrar::FileHash::BLAKE2(_) => true,
            _ => false,
        };
        assert!(file_hash_is_blake2);
        assert_eq!(format!("{:x}", entry.file_crc), "17d406a0c5abab4087a1350b325bf22c90648042f2a032e496313d8bbc7f105e");
    }

}

#[cfg(feature="checksum")]
#[test]
fn test_crc32_checksum() {
    let archive = Archive::new("./data/crc32.rar").open_for_listing().unwrap();
    for entry in archive.into_iter() {
        let entry = entry.unwrap();
        assert!(entry.is_file());
        let file_hash_is_crc32 = match entry.file_crc {
            unrar::FileHash::CRC32(_) => true,
            _ => false,
        };
        assert!(file_hash_is_crc32);
        assert_eq!(format!("{:X}", entry.file_crc), "6E038AF3");
    }
}

#[cfg(not(feature="checksum"))]
#[test]
fn test_crc32_checksum() {
    let archive = Archive::new("./data/crc32.rar").open_for_listing().unwrap();
    for entry in archive.into_iter() {
        let entry = entry.unwrap();
        assert!(entry.is_file());
        assert_eq!(format!("{:X}", entry.file_crc), "6E038AF3");
    }
}
