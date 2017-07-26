use std::io;

use futures::{Async, Future, Poll};
use futures::future::Flatten;

use super::ioc_instance;
use super::ioc_instance::IocInstance;
use super::line_protocol::LineProtocol;
use super::mock_server;
use super::mock_server::MockServerStart;

error_chain! {
    links {
        IocError(ioc_instance::Error, ioc_instance::ErrorKind);
        ServerError(mock_server::Error, mock_server::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
    }

    errors {
        NoIocShellInput {
            description("spawned IOC has no shell input")
        }
    }
}

pub struct IocTest {
    server: Flatten<MockServerStart<LineProtocol>>,
    ioc: IocInstance,
}

impl IocTest {
    pub fn new(
        ioc: IocInstance,
        server: Flatten<MockServerStart<LineProtocol>>,
    ) -> Result<Self> {
        Ok(Self { ioc, server })
    }

    fn poll_ioc(&mut self) -> Poll<(), Error> {
        match self.ioc.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => Err(error.into()),
        }
    }

    fn kill_ioc(&mut self) -> Poll<(), Error> {
        match self.ioc.kill() {
            Ok(_) => Ok(Async::Ready(())),
            Err(error) => Err(error.into()),
        }
    }
}

impl Future for IocTest {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let poll_result = self.server.poll();

        match poll_result {
            Ok(Async::Ready(_)) => self.kill_ioc(),
            Ok(Async::NotReady) => self.poll_ioc(),
            Err(error) => Err(error.into()),
        }
    }
}
