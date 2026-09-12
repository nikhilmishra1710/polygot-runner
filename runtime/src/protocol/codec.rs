use std::io::{self, Read, Write};

use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error};

use super::framing::{read_frame, write_frame};

pub fn send<T: Serialize>(writer: &mut impl Write, value: &T) -> io::Result<()> {
    debug!("encoding value for sending");
    let payload = serde_json::to_vec(value).map_err(|e| {
        error!("failed to serialize value: {}", e);
        io::Error::other(e.to_string())
    })?;

    debug!("sending payload of {} bytes", payload.len());
    if let Err(e) = write_frame(writer, &payload) {
        error!("failed to write frame: {}", e);
        return Err(e);
    }
    debug!("frame sent successfully");
    Ok(())
}

pub fn receive<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<T> {
    debug!("reading frame");
    let payload = match read_frame(reader) {
        Ok(payload) => payload,
        Err(e) => {
            error!("failed to read frame: {}", e);
            return Err(e);
        }
    };
    debug!("received payload of {} bytes", payload.len());

    debug!("deserializing payload");
    let value = serde_json::from_slice(&payload).map_err(|e| {
        error!("failed to deserialize payload: {}", e);
        io::Error::new(io::ErrorKind::InvalidData, e.to_string())
    })?;
    debug!("deserialization successful");
    Ok(value)
}
