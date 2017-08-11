use std::fmt::Display;
use std::hash::Hash;
use std::net::SocketAddr;

use futures::IntoFuture;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ServerProto;

use super::errors::{Error, Result};
use super::ioc_test::IocTest;
use super::super::ioc::IocSpawn;
use super::super::mock_server;
use super::super::mock_server::MockServer;
use super::super::mock_service::When;
use super::super::test_result::TestResult;

pub struct IocTestSetup<P>
where
    P: ServerProto<TcpStream>,
{
    name: String,
    handle: Handle,
    server: MockServer<P>,
    ip_port: u16,
    ioc_variables_to_set: Vec<(String, String)>,
}

impl<P> IocTestSetup<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    pub fn new(handle: Handle, protocol: P, ip_port: u16) -> Result<Self> {
        let address = SocketAddr::new("0.0.0.0".parse()?, ip_port);
        let server = MockServer::new(address, protocol);

        Ok(Self {
            handle,
            server,
            ip_port,
            ioc_variables_to_set: Vec::new(),
            name: String::from("Unnamed IOC test"),
        })
    }

    pub fn name(&mut self, name: &str) {
        self.name = String::from(name);
    }

    pub fn when<A>(&mut self, request: A) -> When<P::Request, P::Response>
    where
        A: Into<<P as ServerProto<TcpStream>>::Request>,
    {
        self.server.when(request)
    }

    pub fn verify<A>(&mut self, request: A)
    where
        A: Into<<P as ServerProto<TcpStream>>::Request>,
    {
        self.server.verify(request);
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        let name = String::from(name);
        let value = String::from(value);

        self.ioc_variables_to_set.push((name, value));
    }
}

impl<P> IntoFuture for IocTestSetup<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    type Future = IocTest<P>;
    type Item = TestResult<Error>;
    type Error = ();

    fn into_future(self) -> Self::Future {
        let ioc = IocSpawn::new(self.handle.clone(), self.ip_port);
        let server = self.server.start(self.handle);

        IocTest::new(self.name, ioc, server, self.ioc_variables_to_set)
    }
}
