use std::io::{self, Read, Write};
use tracing::{debug, error};

pub fn write_frame<W: Write>(
    writer: &mut W,
    payload: &[u8],
) -> io::Result<()> {
    debug!("Writing frame with payload length: {}", payload.len());
    let len = u32::try_from(payload.len())
        .map_err(|e| {
            error!("Failed to convert payload length to u32: {}", e);
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "payload too large",
            )
        })?;

    debug!("Writing frame length: {}", len);
    if let Err(e) = writer.write_all(&len.to_be_bytes()) {
        error!("Failed to write frame length: {}", e);
        return Err(e);
    }
    debug!("Writing frame payload");
    if let Err(e) = writer.write_all(payload) {
        error!("Failed to write frame payload: {}", e);
        return Err(e);
    }
    debug!("Flushing writer");
    if let Err(e) = writer.flush() {
        error!("Failed to flush writer: {}", e);
        return Err(e);
    }
    debug!("Frame written successfully");
    Ok(())
}

pub fn read_frame<R: Read>(
    reader: &mut R,
) -> io::Result<Vec<u8>> {
    debug!("Reading frame length");
    let mut length = [0u8; 4];

    if let Err(e) = reader.read_exact(&mut length) {
        error!("Failed to read frame length: {}", e);
        return Err(e);
    }

    let length = u32::from_be_bytes(length) as usize;
    debug!("Frame length: {}", length);

    // Prevent a malicious client from requesting an enormous allocation.
    const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

    if length > MAX_FRAME_SIZE {
        error!("Frame too large: {} (max: {})", length, MAX_FRAME_SIZE);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }

    debug!("Allocating payload buffer of size: {}", length);
    let mut payload = vec![0u8; length];

    debug!("Reading frame payload");
    if let Err(e) = reader.read_exact(&mut payload) {
        error!("Failed to read frame payload: {}", e);
        return Err(e);
    }
    debug!("Frame payload read successfully");
    Ok(payload)
}