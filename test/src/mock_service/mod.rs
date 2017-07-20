mod errors;
mod expected_request;
mod handle_request;
mod mock_service;

use std::fmt::Display;
use std::io;

use tokio_service::NewService;

pub use self::errors::{Error, ErrorKind, Result};
use self::expected_request::ExpectedRequest;
pub use self::handle_request::HandleRequest;
pub use self::mock_service::MockService;

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
