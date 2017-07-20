use std::collections::VecDeque;
use std::fmt::Display;
use std::io;
use std::sync::{Arc, Mutex};

use futures::{Async, Future, Poll};

use super::errors::{Error, ErrorKind, Result};
use super::expected_request::ExpectedRequest;

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
    pub fn new(
        request: A,
        expected_requests: Arc<Mutex<VecDeque<ExpectedRequest<A, B>>>>,
    ) -> Self {
        Self {
            request,
            expected_requests,
        }
    }

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
