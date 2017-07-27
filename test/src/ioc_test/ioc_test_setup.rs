use std::net::SocketAddr;

use futures::IntoFuture;
use tokio_core::reactor::Handle;

use super::errors::{Error, Result};
use super::ioc_test_start_ioc::IocTestStartIoc;
use super::ioc_test_start::IocTestStart;
use super::super::ioc::IocSpawn;
use super::super::line_protocol::LineProtocol;
use super::super::mock_server::MockServer;
use super::super::mock_service::When;

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
}

impl IntoFuture for IocTestSetup {
    type Future = IocTestStart;
    type Item = IocTestStartIoc;
    type Error = Error;

    fn into_future(self) -> Self::Future {
        let ioc = IocSpawn::new(self.handle.clone(), self.ip_port);
        let server = self.server.start(self.handle);

        IocTestStart::new(ioc, server, self.ioc_variables_to_set)
    }
}
