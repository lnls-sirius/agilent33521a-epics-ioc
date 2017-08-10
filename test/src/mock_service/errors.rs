use std::io;
use std::sync::PoisonError;

error_chain! {
    errors {
        ExpectedRequestQueueAccess {
            description("failed to access expected requests queue")
        }

        NoRequests {
            description("no requests received")
        }

        MissingRequest(request: String) {
            description("expected request not received")
            display("expected request '{}' not received", request)
        }

        UnexpectedRequest(request: String) {
            description("received an unexpected request")
            display("received an unexpected request: '{}'", request)
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        ErrorKind::ExpectedRequestQueueAccess.into()
    }
}

impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.to_string())
    }
}
