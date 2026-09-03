use std::io::{self, Read, Write};

pub fn write_frame<W: Write>(
    writer: &mut W,
    payload: &[u8],
) -> io::Result<()> {
    let len = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(
            io::ErrorKind::InvalidInput,
            "payload too large",
        ))?;

    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;

    Ok(())
}

pub fn read_frame<R: Read>(
    reader: &mut R,
) -> io::Result<Vec<u8>> {
    let mut length = [0u8; 4];

    reader.read_exact(&mut length)?;

    let length = u32::from_be_bytes(length) as usize;

    // Prevent a malicious client from requesting an enormous allocation.
    const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

    if length > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }

    let mut payload = vec![0u8; length];

    reader.read_exact(&mut payload)?;

    Ok(payload)
}