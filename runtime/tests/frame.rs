use runtime_worker::protocol::{read_frame, write_frame};

#[test]
fn frame_round_trip() {
    let payload = b"hello worker";

    let mut buffer = Vec::new();

    write_frame(&mut buffer, payload).unwrap();

    let decoded = read_frame(&mut buffer.as_slice()).unwrap();

    assert_eq!(decoded, payload);
}

#[test]
fn multiple_frames_round_trip() {
    let mut buffer = Vec::new();

    write_frame(&mut buffer, b"one").unwrap();
    write_frame(&mut buffer, b"two").unwrap();

    let mut reader = buffer.as_slice();

    assert_eq!(read_frame(&mut reader).unwrap(), b"one");
    assert_eq!(read_frame(&mut reader).unwrap(), b"two");
}

#[test]
fn oversized_frame_is_rejected() {
    let length = (16 * 1024 * 1024 + 1) as u32;

    let bytes = length.to_be_bytes();

    let result = read_frame(&mut bytes.as_slice());

    assert!(result.is_err());
}
