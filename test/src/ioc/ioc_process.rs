use std::process::ExitStatus;

use bytes::{Bytes, BytesMut, IntoBuf};
use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend};
use tokio_io::AsyncWrite;
use tokio_process::{Child, ChildStdin};

use super::errors::{Error, ErrorKind, Result};

#[derive(Debug)]
pub struct IocProcess {
    process: Child,
    input: ChildStdin,
    input_buffer: BytesMut,
}

impl IocProcess {
    pub fn new(mut process: Child) -> Result<Self> {
        let no_input_error: Error = ErrorKind::IocStdinAccessError.into();
        let input = process.stdin().take().ok_or(no_input_error)?;

        Ok(Self {
            process,
            input,
            input_buffer: BytesMut::new(),
        })
    }

    pub fn kill(&mut self) -> Result<()> {
        Ok(self.process.kill()?)
    }
}

impl Future for IocProcess {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_complete()?;

        Ok(self.process.poll()?)
    }
}

impl Sink for IocProcess {
    type SinkItem = Bytes;
    type SinkError = Error;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.input_buffer.extend(item);

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        if !self.input_buffer.is_empty() {
            let bytes_written = {
                let ref buffer = self.input_buffer;
                let mut buffer = buffer.into_buf();

                try_ready!(self.input.write_buf(&mut buffer))
            };

            self.input_buffer.split_to(bytes_written);
        }

        Ok(Async::Ready(()))
    }
}
