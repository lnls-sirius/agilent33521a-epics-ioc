use futures::{Async, Future, Poll};
use futures::future::Flatten;

use super::errors::{Error, Result};
use super::super::ioc::IocInstance;
use super::super::line_protocol::LineProtocol;
use super::super::mock_server::MockServerStart;

pub struct IocTest {
    server: Flatten<Flatten<MockServerStart<LineProtocol>>>,
    ioc: IocInstance,
}

impl IocTest {
    pub fn new(
        ioc: IocInstance,
        server: Flatten<Flatten<MockServerStart<LineProtocol>>>,
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
        self.ioc.kill();

        self.poll_ioc()
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
