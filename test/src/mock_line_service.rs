use std::collections::VecDeque;
use std::io;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex, PoisonError};

use futures::{Async, Future, Poll};
use tokio_service::{NewService, Service};

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
            description("unexpected request after processing all expected \
                         requests")
            display("received an unexpected request after processing all \
                     expected requests: '{}'", request)
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

impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.to_string())
    }
}

#[derive(Clone, Default)]
struct ExpectedRequest {
    request: String,
    response: String,
}

pub struct MockLineServiceFactory {
    expected_requests: Vec<ExpectedRequest>,
}

impl MockLineServiceFactory {
    pub fn new() -> Self {
        Self {
            expected_requests: Vec::new(),
        }
    }

    pub fn expect(&mut self, request: &str, response: &str) {
        let request = String::from(request);
        let response = String::from(response);
        let expected_request = ExpectedRequest { request, response };

        self.expected_requests.push(expected_request);
    }
}

impl NewService for MockLineServiceFactory {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Instance = MockLineService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let requests = self.expected_requests.clone();

        Ok(Self::Instance::with_expected_requests(requests))
    }
}

pub struct MockLineService {
    expected_requests: Arc<Mutex<VecDeque<ExpectedRequest>>>,
}

impl MockLineService {
    fn with_expected_requests(expected_requests: Vec<ExpectedRequest>) -> Self {
        let expected_requests = VecDeque::from_iter(expected_requests);

        Self {
            expected_requests: Arc::new(Mutex::new(expected_requests)),
        }
    }

    pub fn has_finished(&self) -> Result<bool> {
        Ok(self.expected_requests.lock()?.is_empty())
    }
}

pub struct HandleRequest {
    request: String,
    expected_requests: Arc<Mutex<VecDeque<ExpectedRequest>>>,
}

impl HandleRequest {
    fn handle_request(&self) -> Poll<String, Error> {
        let mut expected_requests = self.expected_requests.lock()?;
        let expected = self.get_next_expected_request(&mut expected_requests)?;

        if expected.request == self.request {
            self.reply_to_request(&expected.response)
        } else {
            self.incorrect_request(&expected.request)
        }
    }

    fn get_next_expected_request(
        &self,
        expected_requests: &mut VecDeque<ExpectedRequest>,
    ) -> Result<ExpectedRequest> {
        match expected_requests.pop_front() {
            Some(expected_request) => Ok(expected_request),
            None => self.unexpected_request(),
        }
    }

    fn reply_to_request(&self, response: &str) -> Poll<String, Error> {
        Ok(Async::Ready(String::from(response)))
    }

    fn unexpected_request(&self) -> Result<ExpectedRequest> {
        let request = self.request.clone();

        Err(ErrorKind::UnexpectedRequest(request).into())
    }

    fn incorrect_request(&self, expected_request: &str) -> Poll<String, Error> {
        let received = self.request.clone();
        let expected = String::from(expected_request);

        Err(ErrorKind::IncorrectRequest(received, expected).into())
    }
}

impl Future for HandleRequest {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.handle_request().map_err(|error| error.into())
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
        }
    }
}
