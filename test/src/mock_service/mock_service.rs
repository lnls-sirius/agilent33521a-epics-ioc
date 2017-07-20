use std::collections::VecDeque;
use std::fmt::Display;
use std::io;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};

use tokio_service::Service;

use super::errors::Result;
use super::expected_request::ExpectedRequest;
use super::handle_request::HandleRequest;

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
    pub fn with_expected_requests(
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
        HandleRequest::new(request, self.expected_requests.clone())
    }
}
