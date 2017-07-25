use std::io;
use std::process::{Command, ExitStatus, Stdio};

use bytes::{BytesMut, IntoBuf};
use futures::{Async, Future, Poll};
use tokio_core::reactor::Handle;
use tokio_io::AsyncWrite;
use tokio_process::{Child, CommandExt};

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        IocStdinAccessError {
            description("failed to access child IOC process standard input")
        }
    }
}

pub struct IocInstance {
    process: Child,
    iocsh_commands: BytesMut,
}

impl IocInstance {
    pub fn new(handle: &Handle, ip_port: u16) -> Result<Self> {
        let process = Command::new("/project/iocBoot/iocagilent33521a/run.sh")
            .env("IPADDR", "127.0.0.1")
            .env("IPPORT", ip_port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn_async(handle)?;

        Ok(Self {
            process,
            iocsh_commands: BytesMut::new(),
        })
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        let command = format!("dbpf {} {}\n", name, value);

        self.iocsh_commands.extend(command.as_bytes());
    }

    pub fn kill(&mut self) -> Result<()> {
        self.process.kill()?;

        Ok(())
    }

    fn try_to_write_iocsh_commands(&mut self) -> Result<()> {
        if !self.iocsh_commands.is_empty() {
            if let &mut Some(ref mut process_input) = self.process.stdin() {
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
