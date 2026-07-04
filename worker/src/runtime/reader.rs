use std::{
    io::{BufReader, Read},
    sync::mpsc,
    thread::{self, JoinHandle},
};

use crate::runtime::{ExecutionEvent, Stream};

const READ_BUFFER_SIZE: usize = 8 * 1024;

pub fn spawn_reader<R>(
    reader: R,
    stream: Stream,
    sender: mpsc::Sender<ExecutionEvent>,
) -> JoinHandle<std::io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut buffer = [0u8; READ_BUFFER_SIZE];

        loop {
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            sender
                .send(ExecutionEvent {
                    stream: stream.clone(),
                    bytes: buffer[..bytes_read].to_vec(),
                })
                .map_err(|_| std::io::Error::other("event receiver dropped"))?;
        }

        Ok(())
    })
}
