use std::io;
use std::net::{AddrParseError, SocketAddr};

use futures::{Async, Future, Poll};
use tokio_core::reactor::Handle;

use super::ioc_instance;
use super::ioc_instance::IocInstance;
use super::line_protocol::LineProtocol;
use super::mock_server;
use super::mock_server::{MockServer, MockServerFuture};
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

pub struct IocTest {
    handle: Handle,
    server: MockServer<LineProtocol>,
    server_future: Option<MockServerFuture<LineProtocol>>,
    ioc: IocInstance,
}

impl IocTest {
    pub fn new(handle: Handle, ip_port: u16) -> Result<Self> {
        let ioc = IocInstance::new(&handle, ip_port)?;
        let address = SocketAddr::new("0.0.0.0".parse()?, ip_port);
        let protocol = LineProtocol::with_separator('\n' as u8);
        let mut server = MockServer::new(address, protocol);

        Self::setup_initial_request_response_map(&mut server);

        Ok(Self {
            ioc,
            handle,
            server,
            server_future: None,
        })
    }

    pub fn when<A>(&mut self, request: A) -> When<String, String>
    where
        A: Into<String>,
    {
        self.server.when(request)
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.ioc.set_variable(name, value);
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

    fn maybe_start_server(
        &mut self,
    ) -> Result<&mut MockServerFuture<LineProtocol>> {
        if self.server_future.is_none() {
            self.start_server();
        }

        if let Some(ref mut server_future) = self.server_future {
            Ok(server_future)
        } else {
            panic!("Server start-up failed but didn't produce an error");
        }
    }

    fn start_server(&mut self) {
        let server_future = self.server.serve_with_handle(self.handle.clone());

        self.server_future = Some(server_future);
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
        let poll_result = self.maybe_start_server()?.poll();

        match poll_result {
            Ok(Async::Ready(_)) => self.kill_ioc(),
            Ok(Async::NotReady) => self.poll_ioc(),
            Err(error) => Err(error.into()),
        }
    }
}
