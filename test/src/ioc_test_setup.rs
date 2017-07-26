use std::io;
use std::net::{AddrParseError, SocketAddr};

use futures::{Future, IntoFuture};
use tokio_core::reactor::Handle;

use super::ioc_instance;
use super::ioc_instance::IocInstance;
use super::ioc_test;
use super::ioc_test::IocTest;
use super::line_protocol::LineProtocol;
use super::mock_server;
use super::mock_server::MockServer;
use super::mock_service::When;

error_chain! {
    links {
        IocError(ioc_instance::Error, ioc_instance::ErrorKind);
        ServerError(mock_server::Error, mock_server::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        InvalidAddress(AddrParseError);
    }

    errors {
        NoIocShellInput {
            description("spawned IOC has no shell input")
        }
    }
}

pub struct IocTestSetup {
    handle: Handle,
    server: MockServer<LineProtocol>,
    ip_port: u16,
    ioc_variables_to_set: Vec<(String, String)>,
}

impl IocTestSetup {
    pub fn new(handle: Handle, ip_port: u16) -> Result<Self> {
        let address = SocketAddr::new("0.0.0.0".parse()?, ip_port);
        let protocol = LineProtocol::with_separator('\n' as u8);
        let mut server = MockServer::new(address, protocol);

        Self::setup_initial_request_response_map(&mut server);

        Ok(Self {
            handle,
            server,
            ip_port,
            ioc_variables_to_set: Vec::new(),
        })
    }

    pub fn when<A>(&mut self, request: A) -> When<String, String>
    where
        A: Into<String>,
    {
        self.server.when(request)
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        let name = String::from(name);
        let value = String::from(value);

        self.ioc_variables_to_set.push((name, value));
    }

    fn setup_initial_request_response_map(
        server: &mut MockServer<LineProtocol>,
    ) {
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

    fn build_ioc_instance(&self) -> IocInstance {
        let mut ioc = IocInstance::new(&self.handle, self.ip_port);

        for &(ref name, ref value) in self.ioc_variables_to_set.iter() {
            ioc.set_variable(&name, &value);
        }

        ioc
    }
}

impl IntoFuture for IocTestSetup {
    type Future = IocTest;
    type Item = ();
    type Error = ioc_test::Error;

    fn into_future(self) -> Self::Future {
        let ioc = self.build_ioc_instance();
        let server = self.server.start(self.handle).flatten();

        IocTest::new(ioc, server).unwrap()
    }
}