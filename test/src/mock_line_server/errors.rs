use std::io;
use std::net::AddrParseError;

use super::super::mock_line_service;

error_chain! {
    links {
        ServiceError(mock_line_service::Error, mock_line_service::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        InvalidAddressToBindTo(AddrParseError);
    }

    errors {
        FailedToReceiveConnection {
            description("failed to receive a connection")
        }

        ActiveStatusHasNoPollEquivalent {
            description("active server status means processing hasn't finished")
        }
    }
}
