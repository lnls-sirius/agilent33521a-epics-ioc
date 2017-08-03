use std::fmt::Display;
use std::hash::Hash;

use futures::{Async, Future, Poll};
use futures::future::Flatten;
use tokio_core::net::TcpStream;
use tokio_proto::pipeline::ServerProto;

use super::errors::Error;
use super::super::ioc::IocInstance;
use super::super::mock_server::ListeningMockServer;

pub struct IocTest<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
{
    server: Flatten<ListeningMockServer<P>>,
    ioc: IocInstance,
}

impl<P> IocTest<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
{
    pub fn new(
        ioc: IocInstance,
        server: Flatten<ListeningMockServer<P>>,
    ) -> Self {
        Self { ioc, server }
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

impl<P> Future for IocTest<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
{
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
