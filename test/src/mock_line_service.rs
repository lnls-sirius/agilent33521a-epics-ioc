use std::io;
use std::sync::{Arc, Mutex, PoisonError};

use futures::{Async, Future, Poll};
use tokio_service::Service;

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
            description("unexpected request after processing all expected requests")
            display("received an unexpected request after processing all expected requests: '{}'", request)
        }

        IncorrectRequest(request: String, expected: String) {
            description("incorrect request received")
            display("received incorrect request '{}' while expecting for '{}'",
                    request, expected)
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        ErrorKind::ExpectedRequestQueueAccess.into()
    }
}

struct ExpectedRequest {
    request: String,
    response: String,
}

type ExpectedRequestQueue = Arc<Mutex<Vec<ExpectedRequest>>>;
type MockLineServiceStatus = Arc<Mutex<Result<()>>>;

pub struct MockLineService {
    expected_requests: ExpectedRequestQueue,
    status: MockLineServiceStatus,
}

impl MockLineService {
    pub fn new() -> Self {
        Self {
            expected_requests: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(Mutex::new(Ok(()))),
        }
    }

    pub fn expect(&mut self, request: String, response: String) -> Result<()> {
        self.queue_expected_request(request, response)
            .and_then(|_| self.reset_status())
    }

    fn queue_expected_request(&mut self, request: String, response: String)
                              -> Result<()> {
        let expected_request = ExpectedRequest { request, response };

        self.expected_requests.lock()?.push(expected_request);

        Ok(())
    }

    fn reset_status(&mut self) -> Result<()> {
        let mut status = self.status.lock()?;

        *status = Err(ErrorKind::NoRequests.into());

        Ok(())
    }
}

pub struct HandleRequest {
    request: String,
    expected_requests: ExpectedRequestQueue,
    status: MockLineServiceStatus,
}

impl HandleRequest {
    fn handle_request(&self) -> Result<String> {
        let mut expected_requests = self.expected_requests.lock()?;
        let expected = self.get_next_expected_request(&mut expected_requests)?;

        if expected.request == self.request {
            self.update_status(&mut expected_requests)?;
            Ok(expected.response)
        } else {
            Err(ErrorKind::IncorrectRequest(self.request.clone(),
                                            expected.request.clone()).into())
        }
    }

    fn get_next_expected_request(&self,
                                 expected_requests: &mut Vec<ExpectedRequest>)
                                 -> Result<ExpectedRequest> {
        match expected_requests.pop() {
            Some(expected_request) => Ok(expected_request),
            None => Err(ErrorKind::UnexpectedRequest(self.request.clone())
                        .into())
        }
    }

    fn update_status(&self, expected_requests: &mut Vec<ExpectedRequest>)
                     -> Result<()> {
        let mut status = self.status.lock()?;

        *status = match expected_requests.first() {
            Some(next_expected_request) => Err(ErrorKind::MissingRequest(
                    next_expected_request.request.clone()).into()),
            None => Ok(())
        };

        Ok(())
    }
}

impl Future for HandleRequest {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.handle_request() {
            Ok(response) => Ok(Async::Ready(response)),
            Err(error) => Err(io::Error::new(io::ErrorKind::Other,
                                             error.to_string()))
        }
    }
}

impl Service for MockLineService {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = HandleRequest;

    fn call(&self, request: Self::Request) -> Self::Future {
        HandleRequest {
            request,
            expected_requests: self.expected_requests.clone(),
            status: self.status.clone(),
        }
    }
}

impl Future for MockLineService {
    type Item = Result<()>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let status = self.status.lock()?;

        match *status {
            Ok(()) => Ok(Async::Ready(Ok(()))),
            Err(ref error) => match *error.kind() {
                ErrorKind::ExpectedRequestQueueAccess =>
                    Err(ErrorKind::ExpectedRequestQueueAccess.into()),
                ErrorKind::NoRequests | ErrorKind::MissingRequest(_) =>
                    Ok(Async::NotReady),
                ErrorKind::UnexpectedRequest(ref request) =>
                    Err(ErrorKind::UnexpectedRequest(request.clone()).into()),
                ErrorKind::IncorrectRequest(ref request, ref expected) =>
                    Err(ErrorKind::IncorrectRequest(request.clone(),
                                                    expected.clone()).into()),
                ErrorKind::Msg(ref message) =>
                    Err(ErrorKind::Msg(message.clone()).into()),
            }
        }
    }
}
