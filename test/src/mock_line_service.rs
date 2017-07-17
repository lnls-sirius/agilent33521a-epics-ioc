use std::io;
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
        Self { expected_requests: Vec::new() }
    }

    pub fn expect(&mut self, request: String, response: String) {
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

#[derive(Clone)]
enum Status {
    AllRequestsProcessed,
    WaitingForRequest(String),
    UnexpectedExtraRequest(String),
    WrongRequest { received: String, expected: String },
}

impl Status {
    fn from(expected_requests: &Vec<ExpectedRequest>) -> Status {
        let first_expected_request = expected_requests.first();

        if let Some(ref expected_request) = first_expected_request {
            Status::WaitingForRequest(expected_request.request.clone())
        } else {
            Status::AllRequestsProcessed
        }
    }
}

impl<T> Into<Poll<T, Error>> for Status
where T: Default
{
    fn into(self) -> Poll<T, Error> {
        match self {
            Status::AllRequestsProcessed =>
                Ok(Async::Ready(Default::default())),
            Status::WaitingForRequest(_) =>
                Ok(Async::NotReady),
            Status::UnexpectedExtraRequest(request) =>
                Err(ErrorKind::UnexpectedRequest(request).into()),
            Status::WrongRequest { received, expected } =>
                Err(ErrorKind::IncorrectRequest(received, expected).into())
        }
    }
}

pub struct MockLineService {
    expected_requests: Arc<Mutex<Vec<ExpectedRequest>>>,
    status: Arc<Mutex<Status>>,
}

impl MockLineService {
    fn with_expected_requests(expected_requests: Vec<ExpectedRequest>) -> Self {
        let status = Status::from(&expected_requests);

        Self {
            expected_requests: Arc::new(Mutex::new(expected_requests)),
            status: Arc::new(Mutex::new(status)),
        }
    }
}

pub struct HandleRequest {
    request: String,
    expected_requests: Arc<Mutex<Vec<ExpectedRequest>>>,
    status: Arc<Mutex<Status>>,
}

impl HandleRequest {
    fn handle_request(&self) -> Poll<String, Error> {
        let mut expected_requests = self.expected_requests.lock()?;
        let expected = self.get_next_expected_request(&mut expected_requests)?;

        if expected.request == self.request {
            self.reply_to_request(&expected.response, &mut expected_requests)
        } else {
            self.incorrect_request(&expected.request)
        }
    }

    fn get_next_expected_request(&self,
                                 expected_requests: &mut Vec<ExpectedRequest>)
         -> Result<ExpectedRequest>
    {
        match expected_requests.pop() {
            Some(expected_request) => Ok(expected_request),
            None => self.unexpected_request(),
        }
    }

    fn update_status<T: Default>(&self, new_status: Status) -> Poll<T, Error> {
        let mut status = self.status.lock()?;

        *status = new_status.clone();

        new_status.into()
    }

    fn reply_to_request(&self, response: &str,
                        expected_requests: &mut Vec<ExpectedRequest>)
        -> Poll<String, Error>
    {
        self.update_status::<()>(Status::from(expected_requests))
            .and(Ok(Async::Ready(String::from(response))))
    }

    fn unexpected_request<T: Default>(&self) -> Result<T> {
        let request = self.request.clone();

        self.update_status::<()>(Status::UnexpectedExtraRequest(request))
            .and(Ok(Default::default()))
    }

    fn incorrect_request<T: Default>(&self, expected_request: &str)
        -> Poll<T, Error>
    {
        let received = self.request.clone();
        let expected = String::from(expected_request);

        self.update_status(Status::WrongRequest { received, expected })
    }
}

impl Future for HandleRequest {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.handle_request()
            .map_err(|error| error.into())
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
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.status.lock()?.clone().into()
    }
}
