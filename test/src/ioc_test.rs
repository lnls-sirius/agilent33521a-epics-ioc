use std::io;
use std::io::{BufWriter, Write};
use std::net::{AddrParseError, SocketAddr};
use std::process::{Child, Command, Stdio};

use futures::{Async, Future, Poll};
use tokio_core::reactor::Handle;

use super::line_protocol::LineProtocol;
use super::mock_server;
use super::mock_server::{MockServer, MockServerFuture};
use super::mock_service::When;

error_chain! {
    links {
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
    ip_port: u16,
    server: MockServer<LineProtocol>,
    server_future: Option<MockServerFuture<LineProtocol>>,
    ioc: Option<Child>,
    iocsh_commands: Vec<String>,
}

impl IocTest {
    pub fn new(handle: Handle, ip_port: u16) -> Result<Self> {
        let address = SocketAddr::new("0.0.0.0".parse()?, ip_port);
        let protocol = LineProtocol::with_separator('\n' as u8);
        let mut server = MockServer::new(address, protocol);

        Self::setup_initial_request_response_map(&mut server);

        Ok(Self {
            handle,
            ip_port,
            server,
            server_future: None,
            ioc: None,
            iocsh_commands: Vec::new(),
        })
    }

    pub fn when<A>(&mut self, request: A) -> When<String, String>
    where
        A: Into<String>,
    {
        self.server.when(request)
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.iocsh_commands
            .push(format!("dbpf {} {}\n", name, value));
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

    fn start_server(&mut self) -> Result<()> {
        let server_future = self.server.serve_with_handle(self.handle.clone());

        self.server_future = Some(server_future);

        Ok(())
    }

    fn start_ioc(&mut self) -> Result<()> {
        let ioc = Command::new("/project/iocBoot/iocagilent33521a/run.sh")
            .env("IPADDR", "127.0.0.1")
            .env("IPPORT", self.ip_port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()?;

        self.ioc = Some(ioc);

        Ok(())
    }

    fn write_iocsh_commands(&mut self) -> Result<()> {
        if let Some(ref mut ioc) = self.ioc {
            let iocsh = ioc.stdin
                .as_mut()
                .ok_or::<Error>(ErrorKind::NoIocShellInput.into())?;

            let mut iocsh_writer = BufWriter::new(iocsh);

            for command in self.iocsh_commands.iter() {
                iocsh_writer.write(command.as_bytes())?;
            }
        } else {
            panic!(
                "Attempt to write IOC shell commands without running the IOC"
            );
        }

        Ok(())
    }

    fn kill_ioc(&mut self) -> Poll<(), Error> {
        if let Some(ref mut ioc) = self.ioc {
            match ioc.kill() {
                Ok(_) => Ok(Async::Ready(())),
                Err(error) => Err(error.into()),
            }
        } else {
            panic!("Attempt to kill IOC that is not running");
        }
    }
}

impl Future for IocTest {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.ioc.is_none() {
            self.start_ioc()?;
            self.write_iocsh_commands()?;
        }

        if self.server_future.is_none() {
            self.start_server()?;
        }

        let poll_result =
            if let Some(ref mut server_future) = self.server_future {
                server_future.poll()
            } else {
                panic!("Server start-up failed but didn't produce an error");
            };

        match poll_result {
            Ok(Async::Ready(_)) => self.kill_ioc(),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => Err(error.into()),
        }
    }
}
