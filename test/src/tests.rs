use std::io;
use std::io::{BufWriter, Write};
use std::net::{AddrParseError, SocketAddr};
use std::process::{Child, Command, Stdio};

use super::mock_line_server;
use super::mock_line_server::MockLineServer;

error_chain! {
    links {
        ServerError(mock_line_server::Error, mock_line_server::ErrorKind);
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

fn create_mock_server(port: u16) -> Result<MockLineServer> {
    let address = SocketAddr::new("0.0.0.0".parse()?, port);
    let mut server = MockLineServer::new(address);

    for _ in 0..2 {
        server.expect("SOURce1:VOLT?", "1");
        server.expect("SOURce1:FUNCtion:ARBitrary:SRATe?", "1");
        server.expect("SOURce1:FREQuency?", "1");
        server.expect("SOURce1:FUNCtion:NOISe:BANDwidth?", "1");
        server.expect("SOURce1:VOLT:OFFSet?", "1");
        server.expect("SOURce1:PHASe?", "1");
        server.expect("SOURce1:FUNCtion:PRBS:BRATe?", "1");
        server.expect("SOURce1:FUNCtion:PRBS:TRANsition?", "1");
        server.expect("SOURce1:FUNCtion:PULSe:TRANsition:LEADing?", "1");
        server.expect("SOURce1:FUNCtion:PULSe:TRANsition:TRAiling?", "1");
        server.expect("SOURce1:FUNCtion:PULSe:WIDTh?", "1");
        server.expect("SOURce1:FUNCtion:RAMP:SYMMetry?", "1");
        server.expect("SOURce1:FUNCtion:SQUare:DCYCle?", "1");
        server.expect("OUTPut1?", "OFF");
        server.expect("SOURce1:FUNCtion:PRBS:DATA?", "PN7");
        server.expect("SOURce1:FUNCtion?", "SQUare");
        server.expect("SOURce1:FUNCtion:ARBitrary?", "\"DUMMY.FILE\"");
    }

    Ok(server)
}

fn start_ioc(port: u16) -> Result<Child> {
    Command::new("/project/iocBoot/iocagilent33521a/run.sh")
        .env("IPADDR", "127.0.0.1")
        .env("IPPORT", port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| error.into())
}

pub fn test_enable_channel_output() -> Result<()> {
    let port = 55000;
    let mut server = create_mock_server(port)?;

    server.expect("OUTPut1 ON", "");

    start_ioc(port)
        .and_then(|mut ioc| {
            ioc.stdin.as_mut().ok_or(ErrorKind::NoIocShellInput.into())
                .and_then(|ref mut iocsh| {
                    let mut iocsh_writer = BufWriter::new(iocsh);

                    iocsh_writer.write("dbpf channelOutput-Sel ON\n".as_bytes())
                        .map_err(|error| error.into())
                })
                .and_then(|_| server.serve().map_err(|error| error.into()))
                .and(ioc.kill().map_err(|error| error.into()))
        })
}
