use std::mem;
use std::process::ExitStatus;

use futures::{AsyncSink, Future, Poll, Sink};

use super::errors::{Error, ErrorKind, Result};
use super::ioc_process::IocProcess;

pub struct IocInstance {
    process: Result<IocProcess>,
}

impl IocInstance {
    pub fn new(process: IocProcess) -> Self {
        Self {
            process: Ok(process),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        let set_variable_error = Err(ErrorKind::SettingIocVariable.into());
        let write_error = Err(ErrorKind::IocWriteError.into());

        let old_process_value =
            mem::replace(&mut self.process, set_variable_error);

        let new_process_value = if let Ok(mut process) = old_process_value {
            let command = format!("dbpf {} {}\n", name, value);

            match process.start_send(command.into()) {
                Ok(AsyncSink::Ready) => Ok(process),
                Ok(AsyncSink::NotReady(_)) => write_error,
                Err(error) => Err(error.into()),
            }
        } else {
            old_process_value
        };

        let _ = mem::replace(&mut self.process, new_process_value);
    }

    pub fn kill(&mut self) {
        let error = Err(ErrorKind::KillingIoc.into());
        let old_process_value = mem::replace(&mut self.process, error);

        let new_process_value = if let Ok(mut process) = old_process_value {
            let kill_result = process.kill();
            let next_access_error = ErrorKind::KilledIoc(process);

            match kill_result {
                Ok(()) => Err(next_access_error.into()),
                Err(error) => Err(error.into()),
            }
        } else {
            old_process_value
        };

        let _ = mem::replace(&mut self.process, new_process_value);
    }
}

impl Future for IocInstance {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let poll_error = Err(ErrorKind::IocInstancePolledAfterEnd.into());
        let old_process_value = mem::replace(&mut self.process, poll_error);

        let (poll_result, new_process_value) = match old_process_value {
            Ok(mut process) => (process.poll(), Ok(process)),
            Err(Error(ErrorKind::KilledIoc(mut process), _)) => {
                (process.poll(), Err(ErrorKind::KilledIoc(process).into()))
            }
            Err(error) => return Err(error),
        };

        let _ = mem::replace(&mut self.process, new_process_value);

        poll_result
    }
}
