use std::fmt::Display;
use std::hash::Hash;
use std::net::SocketAddr;

use futures::IntoFuture;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ServerProto;

use super::errors::{Error, Result};
use super::ioc_test_start_ioc::IocTestStartIoc;
use super::ioc_test_start::IocTestStart;
use super::super::ioc::IocSpawn;
use super::super::mock_server;
use super::super::mock_server::MockServer;
use super::super::mock_service::When;

pub struct IocTestSetup<P>
where
    P: ServerProto<TcpStream>,
{
    handle: Handle,
    server: MockServer<P>,
    ip_port: u16,
    ioc_variables_to_set: Vec<(String, String)>,
}

impl<'a, 'b, P> IocTestSetup<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone
        + Display
        + Eq
        + From<&'a str>
        + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone + From<&'b str>,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    pub fn new(handle: Handle, protocol: P, ip_port: u16) -> Result<Self> {
        let address = SocketAddr::new("0.0.0.0".parse()?, ip_port);
        let mut server = MockServer::new(address, protocol);

        Self::setup_initial_request_response_map(&mut server);

        Ok(Self {
            handle,
            server,
            ip_port,
            ioc_variables_to_set: Vec::new(),
        })
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

    fn setup_initial_request_response_map(server: &mut MockServer<P>) {
        request_response_map! { server,
            "OUTPut1?" => "0",
            "SOURce1:VOLT?" => "1",
            "SOURce1:VOLT:OFFSet?" => "1",
            "SOURce1:FREQuency?" => "1",
            "SOURce1:PHASe?" => "1",
            "SOURce1:FUNCtion?" => "SQUare",
            "SOURce1:FUNCtion:ARBitrary?" => "\"DUMMY.FILE\"",
            "SOURce1:FUNCtion:ARBitrary:SRATe?" => "1",
            "SOURce1:FUNCtion:NOISe:BANDwidth?" => "1",
            "SOURce1:FUNCtion:PRBS:BRATe?" => "1",
            "SOURce1:FUNCtion:PRBS:DATA?" => "PN7",
            "SOURce1:FUNCtion:PRBS:TRANsition?" => "1",
            "SOURce1:FUNCtion:PULSe:TRANsition:LEADing?" => "1",
            "SOURce1:FUNCtion:PULSe:TRANsition:TRAiling?" => "1",
            "SOURce1:FUNCtion:PULSe:WIDTh?" => "1",
            "SOURce1:FUNCtion:RAMP:SYMMetry?" => "1",
            "SOURce1:FUNCtion:SQUare:DCYCle?" => "1",
        };
    }
}

impl<P> IntoFuture for IocTestSetup<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    type Future = IocTestStart<P>;
    type Item = IocTestStartIoc<P>;
    type Error = Error;

    fn into_future(self) -> Self::Future {
        let ioc = IocSpawn::new(self.handle.clone(), self.ip_port);
        let server = self.server.start(self.handle);

        IocTestStart::new(ioc, server, self.ioc_variables_to_set)
    }
}
