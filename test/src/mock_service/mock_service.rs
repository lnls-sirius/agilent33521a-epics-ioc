use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::io;
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
    expected_requests: Arc<Mutex<HashMap<A, B>>>,
    requests_to_verify: Arc<Mutex<HashSet<A>>>,
}

impl<A, B> MockService<A, B>
where
    A: Clone + Display + Eq + Hash,
    B: Clone,
{
    pub fn new(
        expected_requests: Vec<ExpectedRequest<A, B>>,
        requests_to_verify: HashSet<A>,
    ) -> Self {
        let number_of_requests = expected_requests.len();
        let mut request_map = HashMap::with_capacity(number_of_requests);

        for expected_request in expected_requests {
            let request = expected_request.request;
            let response = expected_request.response;

            request_map.insert(request, response);
        }

        Self {
            requests_to_verify: Arc::new(Mutex::new(requests_to_verify)),
            expected_requests: Arc::new(Mutex::new(request_map)),
        }
    }

    pub fn has_finished(&self) -> Result<bool> {
        Ok(self.requests_to_verify.lock()?.is_empty())
    }
}

impl<A, B> Service for MockService<A, B>
where
    A: Clone + Display + Eq + Hash,
    B: Clone,
{
    type Request = A;
    type Response = B;
    type Error = io::Error;
    type Future = HandleRequest<A, B>;

    fn call(&self, request: Self::Request) -> Self::Future {
        HandleRequest::new(
            request,
            self.expected_requests.clone(),
            self.requests_to_verify.clone(),
        )
    }
}
