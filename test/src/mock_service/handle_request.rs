use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::io;
use std::sync::{Arc, Mutex};

use futures::{Async, Future, Poll};

use super::errors::{Error, ErrorKind};

pub struct HandleRequest<A, B>
where
    A: Clone + Display + Eq,
    B: Clone,
{
    request: A,
    expected_requests: Arc<Mutex<HashMap<A, B>>>,
    remaining_requests: Arc<Mutex<HashSet<A>>>,
}

impl<A, B> HandleRequest<A, B>
where
    A: Clone + Display + Eq + Hash,
    B: Clone,
{
    pub fn new(
        request: A,
        expected_requests: Arc<Mutex<HashMap<A, B>>>,
        remaining_requests: Arc<Mutex<HashSet<A>>>,
    ) -> Self {
        Self {
            request,
            expected_requests,
            remaining_requests,
        }
    }

    fn handle_request(&self) -> Poll<B, Error> {
        let expected_requests = self.expected_requests.lock()?;

        if let Some(response) = expected_requests.get(&self.request) {
            self.reply_to_request(response.clone())
        } else {
            self.unexpected_request()
        }
    }

    fn reply_to_request(&self, response: B) -> Poll<B, Error> {
        let mut remaining_requests = self.remaining_requests.lock()?;

        remaining_requests.remove(&self.request);

        Ok(Async::Ready(response))
    }

    fn unexpected_request(&self) -> Poll<B, Error> {
        Err(
            ErrorKind::UnexpectedRequest(self.request.to_string()).into(),
        )
    }
}

impl<A, B> Future for HandleRequest<A, B>
where
    A: Clone + Display + Eq + Hash,
    B: Clone,
{
    type Item = B;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.handle_request().map_err(|error| error.into())
    }
}
