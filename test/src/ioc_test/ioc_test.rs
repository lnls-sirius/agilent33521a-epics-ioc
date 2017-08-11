use std::fmt::Display;
use std::hash::Hash;

use futures::{Async, Future, Poll};
use futures::future::Flatten;
use tokio_core::net::TcpStream;
use tokio_proto::pipeline::ServerProto;

use super::errors::Error;
use super::ioc_test_start::IocTestStart;
use super::super::ioc::IocSpawn;
use super::super::mock_server;
use super::super::mock_server::MockServerStart;
use super::super::test_result::TestResult;

pub struct IocTest<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    name: String,
    future: Flatten<Flatten<IocTestStart<P>>>,
}

impl<P> IocTest<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    pub fn new(
        ioc: IocSpawn,
        server: MockServerStart<P>,
        ioc_variables_to_set: Vec<(String, String)>,
    ) -> Self {
        let test_start = IocTestStart::new(ioc, server, ioc_variables_to_set);

        Self {
            name: String::from("IocTest"),
            future: test_start.flatten().flatten(),
        }
    }
}

impl<P> Future for IocTest<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    type Item = TestResult<Error>;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.future.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(())) => {
                Ok(Async::Ready(TestResult::success(self.name.clone())))
            }
            Err(error) => {
                Ok(Async::Ready(TestResult::failure(self.name.clone(), error)))
            }
        }
    }
}
