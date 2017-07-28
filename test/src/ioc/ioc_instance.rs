use std::mem;
use std::process::ExitStatus;

use futures::{AsyncSink, Future, Poll, Sink};

use super::errors::{Error, ErrorKind};
use super::ioc_process::IocProcess;

pub struct IocInstance {
    process: IocProcess,
    error: Option<Error>,
}

impl IocInstance {
    pub fn new(process: IocProcess) -> Self {
        Self {
            process,
            error: None,
        }
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        if self.error.is_none() {
            let command = format!("dbpf {} {}\n", name, value);
            let write_error = ErrorKind::IocWriteError.into();

            self.error = match self.process.start_send(command.into()) {
                Ok(AsyncSink::Ready) => None,
                Ok(AsyncSink::NotReady(_)) => Some(write_error),
                Err(error) => error.into(),
            }
        }
    }

    pub fn kill(&mut self) {
        if self.error.is_none() {
            self.process.kill();
        }
    }
}

impl Future for IocInstance {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let temporary_error = ErrorKind::IocInstancePolledAfterEnd.into();
        let error_status = mem::replace(&mut self.error, Some(temporary_error));

        let (poll_result, new_error_status) = match error_status {
            None => (self.process.poll(), None),
            Some(error) => (
                Err(error),
                Some(ErrorKind::IocInstancePolledAfterEnd.into()),
            ),
        };

        let _temporary_error = mem::replace(&mut self.error, new_error_status);

        poll_result
    }
}
