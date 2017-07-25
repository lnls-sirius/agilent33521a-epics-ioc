use std::io;
use std::mem;
use std::net::{AddrParseError, SocketAddr};

use futures::{Async, Future, Poll};
use futures::future::Flatten;
use tokio_core::reactor::Handle;

use super::ioc_instance;
use super::ioc_instance::IocInstance;
use super::line_protocol::LineProtocol;
use super::mock_server;
use super::mock_server::{MockServer, MockServerStart};
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

enum Server {
    Building(MockServer<LineProtocol>),
    Starting,
    Ready(Flatten<MockServerStart<LineProtocol>>),
}

impl Server {
    fn start_if_necessary(
        &mut self,
        handle: &Handle,
    ) -> &mut Flatten<MockServerStart<LineProtocol>> {
        let started_server = match mem::replace(self, Server::Starting) {
            Server::Building(server) => {
                Server::Ready(server.start(handle.clone()).flatten())
            }
            Server::Ready(server) => Server::Ready(server),
            Server::Starting => panic!("more than one owner of server state"),
        };

        mem::replace(self, started_server);

        match *self {
            Server::Ready(ref mut server) => server,
            _ => unreachable!(),
        }
    }
}

pub struct IocTest {
    handle: Handle,
    server: Server,
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
            server: Server::Building(server),
        })
    }

    pub fn when<A>(&mut self, request: A) -> When<String, String>
    where
        A: Into<String>,
    {
        match self.server {
            Server::Building(ref mut server) => server.when(request),
            _ => panic!("Can't update mock server after it has started"),
        }
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
        let poll_result = self.server.start_if_necessary(&self.handle).poll();

        match poll_result {
            Ok(Async::Ready(_)) => self.kill_ioc(),
            Ok(Async::NotReady) => self.poll_ioc(),
            Err(error) => Err(error.into()),
        }
    }
}
