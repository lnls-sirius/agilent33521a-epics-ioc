use std::io;
use std::net::AddrParseError;

use super::super::ioc;
use super::super::mock_server;

error_chain! {
    links {
        IocError(ioc::Error, ioc::ErrorKind);
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
