use std::mem;
use std::process::ExitStatus;

use bytes::{BytesMut, IntoBuf};
use futures::{Async, Future, Poll};
use tokio_core::reactor::Handle;
use tokio_io::AsyncWrite;
use tokio_process::{Child, ChildStdin};

use super::errors::{Error, ErrorKind, Result};
use super::ioc_spawn::IocSpawn;

enum Process {
    Polling,
    Spawning(IocSpawn),
    Running(Child),
    Killed(Result<Child>),
}

impl Process {
    fn is_running(&self) -> bool {
        match *self {
            Process::Polling => panic!("can't use Process while polling it"),
            Process::Spawning(_) => false,
            Process::Running(_) => true,
            Process::Killed(_) => false,
        }
    }

    fn stdin(&mut self) -> Option<&mut ChildStdin> {
        match *self {
            Process::Polling => panic!("can't use Process while polling it"),
            Process::Spawning(_) => None,
            Process::Running(ref mut process) => {
                if let &mut Some(ref mut input) = process.stdin() {
                    Some(input)
                } else {
                    None
                }
            }
            Process::Killed(_) => None,
        }
    }

    fn kill(&mut self) {
        let error: Error = ErrorKind::IocSpawnCancelled.into();
        let old_state = mem::replace(self, Process::Killed(Err(error)));

        match old_state {
            Process::Polling => panic!("can't use Process while polling it"),
            Process::Spawning(_) => (),
            Process::Running(mut process) => {
                let result: Result<Child> = process
                    .kill()
                    .and(Ok(process))
                    .map_err(|error| error.into());

                mem::replace(self, Process::Killed(result));
            }
            Process::Killed(result) => {
                mem::replace(self, Process::Killed(result));
            }
        }
    }
}

impl Future for Process {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let old_state = mem::replace(self, Process::Polling);

        match old_state {
            Process::Polling => {
                panic!("polled simultaneously by different threads")
            }
            Process::Spawning(mut spawn) => {
                match spawn.poll() {
                    Ok(Async::Ready(mut running_process)) => {
                        let poll_result = running_process.poll();

                        mem::replace(self, Process::Running(running_process));

                        Ok(poll_result?)
                    }
                    Ok(Async::NotReady) => {
                        mem::replace(self, Process::Spawning(spawn));

                        Ok(Async::NotReady)
                    }
                    Err(error) => {
                        mem::replace(self, Process::Spawning(spawn));

                        Err(error.into())
                    }
                }
            }
            Process::Running(mut process) => {
                let poll_result = process.poll();

                mem::replace(self, Process::Running(process));

                Ok(poll_result?)
            }
            Process::Killed(Ok(mut process)) => {
                let poll_result = process.poll();

                mem::replace(self, Process::Killed(Ok(process)));

                Ok(poll_result?)
            }
            Process::Killed(Err(error)) => {
                let new_error = ErrorKind::IocProcessPolledAfterEnd;

                mem::replace(self, Process::Killed(Err(new_error.into())));

                Err(error)
            }
        }
    }
}

pub struct IocInstance {
    process: Process,
    iocsh_commands: BytesMut,
}

impl IocInstance {
    pub fn new(handle: &Handle, ip_port: u16) -> Self {
        let spawn = IocSpawn::new(handle.clone(), ip_port);

        Self {
            process: Process::Spawning(spawn),
            iocsh_commands: BytesMut::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        let command = format!("dbpf {} {}\n", name, value);

        self.iocsh_commands.extend(command.as_bytes());
    }

    pub fn kill(&mut self) {
        self.process.kill();
    }

    fn try_to_write_iocsh_commands(&mut self) -> Result<()> {
        if !self.iocsh_commands.is_empty() && self.process.is_running() {
            if let Some(ref mut process_input) = self.process.stdin() {
                let mut ready_to_write = true;

                while !self.iocsh_commands.is_empty() && ready_to_write {
                    let write_result = {
                        let ref buffer = self.iocsh_commands;
                        let mut buffer = buffer.into_buf();

                        process_input.write_buf(&mut buffer)?
                    };

                    match write_result {
                        Async::Ready(bytes_written) => {
                            self.iocsh_commands.split_to(bytes_written);
                        }
                        Async::NotReady => ready_to_write = false,
                    };
                }
            }
        }

        Ok(())
    }
}

impl Future for IocInstance {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.try_to_write_iocsh_commands()?;

        Ok(self.process.poll()?)
    }
}
