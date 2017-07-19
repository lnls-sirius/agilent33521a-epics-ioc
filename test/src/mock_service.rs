use std::collections::VecDeque;
use std::fmt::Display;
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
struct ExpectedRequest<A, B>
where
    A: Clone,
    B: Clone,
{
    request: A,
    response: B,
}

pub struct MockServiceFactory<A, B>
where
    A: Clone,
    B: Clone,
{
    expected_requests: Vec<ExpectedRequest<A, B>>,
}

impl<A, B> MockServiceFactory<A, B>
where
    A: Clone,
    B: Clone,
{
    pub fn new() -> Self {
        Self {
            expected_requests: Vec::new(),
        }
    }

    pub fn expect(&mut self, request: A, response: B) {
        let expected_request = ExpectedRequest { request, response };

        self.expected_requests.push(expected_request);
    }
}

impl<A, B> NewService for MockServiceFactory<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    type Request = A;
    type Response = B;
    type Error = io::Error;
    type Instance = MockService<A, B>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let requests = self.expected_requests.clone();

        Ok(Self::Instance::with_expected_requests(requests))
    }
}

pub struct MockService<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    expected_requests: Arc<Mutex<VecDeque<ExpectedRequest<A, B>>>>,
}

impl<A, B> MockService<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    fn with_expected_requests(
        expected_requests: Vec<ExpectedRequest<A, B>>,
    ) -> Self {
        let expected_requests = VecDeque::from_iter(expected_requests);

        Self {
            expected_requests: Arc::new(Mutex::new(expected_requests)),
        }
    }

    pub fn has_finished(&self) -> Result<bool> {
        Ok(self.expected_requests.lock()?.is_empty())
    }
}

pub struct HandleRequest<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    request: A,
    expected_requests: Arc<Mutex<VecDeque<ExpectedRequest<A, B>>>>,
}

impl<A, B> HandleRequest<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    fn handle_request(&self) -> Poll<B, Error> {
        let mut expected_requests = self.expected_requests.lock()?;
        let expected = self.get_next_expected_request(&mut expected_requests)?;

        if expected.request == self.request {
            self.reply_to_request(expected.response)
        } else {
            self.incorrect_request(expected.request)
        }
    }

    fn get_next_expected_request(
        &self,
        expected_requests: &mut VecDeque<ExpectedRequest<A, B>>,
    ) -> Result<ExpectedRequest<A, B>> {
        match expected_requests.pop_front() {
            Some(expected_request) => Ok(expected_request),
            None => self.unexpected_request(),
        }
    }

    fn reply_to_request(&self, response: B) -> Poll<B, Error> {
        Ok(Async::Ready(response))
    }

    fn unexpected_request(&self) -> Result<ExpectedRequest<A, B>> {
        Err(
            ErrorKind::UnexpectedRequest(self.request.to_string()).into(),
        )
    }

    fn incorrect_request(&self, expected_request: A) -> Poll<B, Error> {
        let received = self.request.to_string();
        let expected = expected_request.to_string();

        Err(ErrorKind::IncorrectRequest(received, expected).into())
    }
}

impl<A, B> Future for HandleRequest<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    type Item = B;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.handle_request().map_err(|error| error.into())
    }
}

impl<A, B> Service for MockService<A, B>
where
    A: Clone + Display + PartialEq,
    B: Clone,
{
    type Request = A;
    type Response = B;
    type Error = io::Error;
    type Future = HandleRequest<A, B>;

    fn call(&self, request: Self::Request) -> Self::Future {
        HandleRequest {
            request,
            expected_requests: self.expected_requests.clone(),
        }
    }
}
