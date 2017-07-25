use std::io;
use std::io::{BufWriter, Write};
use std::mem;
use std::process::{Child, Command, Stdio};

use bytes::{Buf, BytesMut, IntoBuf};
use futures::{Async, Future, Poll};

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
    pub fn new(ip_port: u16) -> Result<Self> {
        let process = Command::new("/project/iocBoot/iocagilent33521a/run.sh")
            .env("IPADDR", "127.0.0.1")
            .env("IPPORT", ip_port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()?;

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
            let process_input = self.process
                .stdin
                .as_mut()
                .ok_or::<Error>(ErrorKind::IocStdinAccessError.into())?;

            let mut iocsh_writer = BufWriter::new(process_input);
            let iocsh_commands =
                mem::replace(&mut self.iocsh_commands, BytesMut::new());

            iocsh_writer.write_all(iocsh_commands.into_buf().bytes())?;
        }

        Ok(())
    }
}

impl Future for IocInstance {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.try_to_write_iocsh_commands()?;

        Ok(Async::Ready(()))
    }
}
