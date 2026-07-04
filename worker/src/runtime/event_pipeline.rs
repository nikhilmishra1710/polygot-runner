use std::{
    io::{self, Result},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use crate::runtime::{BufferedConsumer, EventConsumer, ExecutionEvent};

pub struct EventPipeline {
    sender: Sender<ExecutionEvent>,
    consumer_handle: JoinHandle<std::io::Result<BufferedConsumer>>,
}

impl EventPipeline {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<ExecutionEvent>();

        let consumer_handle = thread::spawn(move || -> io::Result<BufferedConsumer> {
            let mut consumer = BufferedConsumer::new();

            while let Ok(event) = receiver.recv() {
                consumer.consume(event)?;
            }

            Ok(consumer)
        });

        Self {
            sender,
            consumer_handle,
        }
    }

    pub fn sender(&self) -> Sender<ExecutionEvent> {
        self.sender.clone()
    }

    pub fn finish(self) -> Result<BufferedConsumer> {
        drop(self.sender);

        let consumer = self
            .consumer_handle
            .join()
            .map_err(|_| io::Error::other("consumer thread panicked"))??;

        Ok(consumer)
    }
}
