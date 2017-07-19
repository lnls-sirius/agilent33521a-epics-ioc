use std::io;
use std::net::AddrParseError;

use super::super::mock_service;

error_chain! {
    links {
        ServiceError(mock_service::Error, mock_service::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        InvalidAddressToBindTo(AddrParseError);
    }

    errors {
        FailedToReceiveConnection {
            description("failed to receive a connection")
        }

        FailedToBindConnection {
            description("failed to bind the connection to receive requests")
        }

        ActiveStatusHasNoPollEquivalent {
            description("active server status means processing hasn't finished")
        }
    }
}
