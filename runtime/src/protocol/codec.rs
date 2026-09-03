use std::io::{self, Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::framing::{read_frame, write_frame};

pub fn send<T: Serialize>(writer: &mut impl Write, value: &T) -> io::Result<()> {
    let payload = bincode::serialize(value).map_err(|e| io::Error::other(e.to_string()))?;

    write_frame(writer, &payload)
}

pub fn receive<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<T> {
    let payload = read_frame(reader)?;

    bincode::deserialize(&payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}
