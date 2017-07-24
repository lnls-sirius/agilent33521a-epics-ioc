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
    remaining_requests: Arc<Mutex<HashSet<A>>>,
}

impl<A, B> MockService<A, B>
where
    A: Clone + Display + Eq + Hash,
    B: Clone,
{
    pub fn with_expected_requests(
        expected_requests: Vec<ExpectedRequest<A, B>>,
    ) -> Self {
        let number_of_requests = expected_requests.len();
        let mut request_map = HashMap::with_capacity(number_of_requests);
        let mut remaining_requests = HashSet::with_capacity(number_of_requests);

        for expected_request in expected_requests {
            let request = expected_request.request;
            let response = expected_request.response;

            remaining_requests.insert(request.clone());
            request_map.insert(request, response);
        }

        Self {
            expected_requests: Arc::new(Mutex::new(request_map)),
            remaining_requests: Arc::new(Mutex::new(remaining_requests)),
        }
    }

    pub fn has_finished(&self) -> Result<bool> {
        Ok(self.remaining_requests.lock()?.is_empty())
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
            self.remaining_requests.clone(),
        )
    }
}
