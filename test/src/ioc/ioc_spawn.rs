use std::io;
use std::process::{Command, Stdio};

use futures::{Async, Future, Poll};
use tokio_core::reactor::Handle;
use tokio_process::{Child, CommandExt};

pub struct IocSpawn {
    handle: Handle,
    ip_port: u16,
}

impl IocSpawn {
    pub fn new(handle: Handle, ip_port: u16) -> Self {
        Self { handle, ip_port }
    }
}

impl Future for IocSpawn {
    type Item = Child;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let process = Command::new("/project/iocBoot/iocagilent33521a/run.sh")
            .env("IPADDR", "127.0.0.1")
            .env("IPPORT", self.ip_port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn_async(&self.handle)?;

        Ok(Async::Ready(process))
    }
}
