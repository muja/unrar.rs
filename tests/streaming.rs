use std::io::Read;
use unrar::Archive;

#[test]
fn test_streaming_read_basic() {
    let streaming_bytes = {
        let archive = Archive::new("data/version.rar")
            .open_for_processing()
            .unwrap();
        let header = archive.read_header().unwrap().unwrap();
        let mut reader = header.read_streaming();
        let mut data = Vec::new();
        reader.read_to_end(&mut data).unwrap();
        let _ = reader.finish();
        data
    };

    let read_bytes = Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap()
        .read_header()
        .unwrap()
        .unwrap()
        .read()
        .unwrap()
        .0;

    assert_eq!(streaming_bytes, read_bytes);
    assert_eq!(String::from_utf8(streaming_bytes).unwrap(), "unrar-0.4.0");
}

#[test]
fn test_streaming_read_take_limit() {
    let archive = Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap();
    let header = archive.read_header().unwrap().unwrap();
    let mut reader = header.read_streaming();
    let mut data = Vec::new();
    let bytes_read = (&mut reader).take(5).read_to_end(&mut data).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(&data, b"unrar");
    let _ = reader.finish();
}

#[test]
fn test_streaming_finish_after_partial() {
    let archive = Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap();
    let header = archive.read_header().unwrap().unwrap();
    let mut reader = header.read_streaming();
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"unr");
    let (archive, _decompressed) = reader.finish().unwrap();
    drop(archive);
}

#[test]
fn test_streaming_drop_aborts() {
    use std::time::Instant;
    let start = Instant::now();
    {
        let archive = Archive::new("data/version.rar")
            .open_for_processing()
            .unwrap();
        let header = archive.read_header().unwrap().unwrap();
        let mut reader = header.read_streaming();
        let mut buf = [0u8; 1];
        let _ = reader.read(&mut buf);
    }
    assert!(start.elapsed().as_secs() < 6, "Drop took too long");
}

#[test]
fn test_streaming_immediate_drop() {
    use std::time::Instant;
    let start = Instant::now();
    {
        let archive = Archive::new("data/version.rar")
            .open_for_processing()
            .unwrap();
        let header = archive.read_header().unwrap().unwrap();
        let _reader = header.read_streaming();
    }
    assert!(start.elapsed().as_secs() < 6, "Immediate drop hung");
}

#[test]
fn test_streaming_read_matches_read_to_vec() {
    let mut streaming_archive = Archive::new("data/solid.rar")
        .open_for_processing()
        .unwrap();
    let mut read_archive = Archive::new("data/solid.rar")
        .open_for_processing()
        .unwrap();

    loop {
        let s_header = streaming_archive.read_header().unwrap();
        let r_header = read_archive.read_header().unwrap();
        match (s_header, r_header) {
            (Some(sh), Some(rh)) => {
                let (read_data, next_r) = rh.read().unwrap();
                read_archive = next_r;

                let mut reader = sh.read_streaming();
                let mut streaming_data = Vec::new();
                reader.read_to_end(&mut streaming_data).unwrap();
                reader.set_discard();
                (streaming_archive, _) = reader.finish().unwrap();

                assert_eq!(streaming_data, read_data, "Data mismatch in solid archive");
            }
            (None, None) => break,
            _ => panic!("Archive entry count mismatch"),
        }
    }
}

/// Verifies that calling `set_discard()` on a solid archive after a partial
/// read completes without error and that `finish()` returns a valid archive
/// handle and the final decompressed byte count.
#[test]
fn test_streaming_solid_discard_completes() {
    let archive = Archive::new("data/solid.rar")
        .open_for_processing()
        .unwrap();
    assert!(
        archive.is_solid(),
        "Expected solid.rar to be a solid archive"
    );

    let header = archive.read_header().unwrap().unwrap();
    let entry_size = header.entry().unpacked_size;
    assert!(
        entry_size > 1,
        "Entry must be larger than 1 byte for this test"
    );

    let mut reader = header.read_streaming();
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).unwrap();

    reader.set_discard();

    let (_archive, decompressed) = reader
        .finish()
        .expect("finish() should succeed after set_discard()");
    // After discard-mode finish, decompressed_bytes should reflect the full entry
    assert!(
        decompressed >= entry_size,
        "decompressed_bytes ({}) should be >= entry unpacked_size ({})",
        decompressed,
        entry_size
    );
}

#[test]
fn test_streaming_discard_then_drop() {
    use std::time::Instant;
    let start = Instant::now();
    {
        let archive = Archive::new("data/version.rar")
            .open_for_processing()
            .unwrap();
        let header = archive.read_header().unwrap().unwrap();
        let reader = header.read_streaming();
        reader.set_discard();
    }
    assert!(start.elapsed().as_secs() < 6, "Discard-then-drop hung");
}

#[test]
fn test_streaming_abort_on_limit() {
    let archive = Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap();
    let header = archive.read_header().unwrap().unwrap();
    let entry_size = header.entry().unpacked_size;
    let limit = 5u64;
    assert!(
        entry_size > limit,
        "Entry must be larger than limit for this test"
    );

    let mut reader = header.read_streaming();
    let mut data = Vec::new();
    let bytes_read = (&mut reader).take(limit).read_to_end(&mut data).unwrap();
    assert_eq!(bytes_read as u64, limit);
    assert_eq!(data.len() as u64, limit);
    let (_archive, _) = reader.finish().unwrap();
}

#[test]
fn test_streaming_error_on_encrypted_without_password() {
    let archive = Archive::new("data/crypted.rar")
        .open_for_processing()
        .unwrap();
    let header = archive.read_header().unwrap().unwrap();
    let mut reader = header.read_streaming();
    let mut data = Vec::new();
    let result = reader.read_to_end(&mut data);
    assert!(
        result.is_err(),
        "Reading encrypted entry without password should fail"
    );
    let finish_result = reader.finish();
    assert!(
        finish_result.is_err(),
        "finish() should propagate the error"
    );
}
