use std::{
    io::{BufReader, Read},
    process::ChildStdout,
    thread,
};

use crate::runtime::{BufferedConsumer, EventConsumer, ExecutionEvent};

const READ_BUFFER_SIZE: usize = 8 * 1024;

pub enum Stream {
    Stdout,
    Stderr,
}

pub fn spawn_reader<R>(
    reader: R,
    stream: Stream,
) -> thread::JoinHandle<std::io::Result<BufferedConsumer>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut consumer = BufferedConsumer::new();
        let mut buffer = [0u8; READ_BUFFER_SIZE];

        loop {
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            let event = match stream {
                Stream::Stdout => ExecutionEvent::stdout(buffer[..bytes_read].to_vec()),
                Stream::Stderr => ExecutionEvent::stderr(buffer[..bytes_read].to_vec()),
            };

            consumer.consume(event)?;
        }

        Ok(consumer)
    })
}
