use std::{io, mem};

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use futures::stream::FuturesUnordered;
use tokio_core::net::TcpStream;
use tokio_io::codec::Framed;
use tokio_service::Service;

use super::errors::Error;
use super::status::Status;
use super::super::line_codec::LineCodec;
use super::super::mock_line_service::{HandleRequest, MockLineService};

pub struct ActiveMockLineServer {
    connection: Framed<TcpStream, LineCodec>,
    service: MockLineService,
    live_requests: FuturesUnordered<HandleRequest>,
    live_responses: Vec<String>,
    status: Status,
}

impl ActiveMockLineServer {
    pub fn new(
        connection: Framed<TcpStream, LineCodec>,
        service: MockLineService,
    ) -> Self {
        Self {
            connection,
            service,
            live_requests: FuturesUnordered::new(),
            live_responses: Vec::new(),
            status: Status::Active,
        }
    }

    fn try_to_get_new_request(&mut self) -> &mut Self {
        if self.status.is_active() {
            let new_request = self.connection.poll();

            if let Ok(Async::Ready(Some(request))) = new_request {
                self.live_requests.push(self.service.call(request));
            } else {
                self.status.update(new_request);
            }
        }

        self
    }

    fn try_to_get_new_response(&mut self) -> &mut Self {
        if self.status.is_active() {
            let maybe_response = self.live_requests.poll();

            if let Ok(Async::Ready(Some(response))) = maybe_response {
                self.live_responses.push(response);
            } else {
                self.status.update(maybe_response);
            }
        }

        self
    }

    fn try_to_send_responses(&mut self) -> &mut Self {
        if self.status.is_active() {
            let first_failed_send = self.send_responses_while_possible();

            if let Some((index, status)) = first_failed_send {
                self.live_responses.drain(0..index);
                self.status.update(status);
            } else {
                self.live_responses.clear();
            }
        }

        self
    }

    fn send_responses_while_possible(
        &mut self,
    ) -> Option<(usize, StartSend<String, io::Error>)> {
        let connection = &mut self.connection;

        self.live_responses
            .iter()
            .map(|response| connection.start_send(response.clone()))
            .enumerate()
            .find(|&(_, ref status)| match *status {
                Ok(AsyncSink::Ready) => false,
                Ok(AsyncSink::NotReady(_)) => true,
                Err(_) => true,
            })
    }

    fn try_to_flush_responses(&mut self) -> &mut Self {
        if self.status.is_active() {
            self.status.update(self.connection.poll_complete());
        }

        self
    }

    fn check_if_finished(&mut self) {
        if self.status.is_active() {
            let no_pending_requests = self.live_requests.is_empty();
            let no_pending_responses = self.live_responses.is_empty();

            if no_pending_requests && no_pending_responses {
                self.status = match self.service.has_finished() {
                    Ok(true) => Status::Finished,
                    Ok(false) => Status::Active,
                    Err(error) => Status::Error(error.into()),
                }
            }
        }
    }

    fn poll_status(&mut self) -> Poll<(), Error> {
        let resulting_status = mem::replace(&mut self.status, Status::Active);

        resulting_status.into()
    }
}

impl Future for ActiveMockLineServer {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while self.status.is_active() {
            self.try_to_get_new_request()
                .try_to_get_new_response()
                .try_to_send_responses()
                .try_to_flush_responses()
                .check_if_finished();
        }

        self.poll_status()
    }
}
