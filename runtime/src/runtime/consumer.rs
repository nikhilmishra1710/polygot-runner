use std::io;

use super::event::{ExecutionEvent, Stream};

pub trait EventConsumer: Send {
    fn consume(&mut self, event: ExecutionEvent) -> io::Result<()>;
}

#[derive(Default)]
pub struct BufferedConsumer {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl BufferedConsumer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_output(self) -> (Vec<u8>, Vec<u8>) {
        (self.stdout, self.stderr)
    }
}

impl EventConsumer for BufferedConsumer {
    fn consume(&mut self, event: ExecutionEvent) -> io::Result<()> {
        match event.stream {
            Stream::Stdout => self.stdout.extend(event.bytes),
            Stream::Stderr => self.stderr.extend(event.bytes),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ExecutionEvent;

    #[test]
    fn buffers_stdout() {
        let mut consumer = BufferedConsumer::new();

        consumer
            .consume(ExecutionEvent::stdout(b"hello".to_vec()))
            .unwrap();

        let (stdout, stderr) = consumer.into_output();

        assert_eq!(stdout, b"hello");
        assert!(stderr.is_empty());
    }

    #[test]
    fn buffers_stderr() {
        let mut consumer = BufferedConsumer::new();

        consumer
            .consume(ExecutionEvent::stderr(b"oops".to_vec()))
            .unwrap();

        let (stdout, stderr) = consumer.into_output();

        assert!(stdout.is_empty());
        assert_eq!(stderr, b"oops");
    }
}
