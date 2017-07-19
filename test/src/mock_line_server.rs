use std::{io, mem};
use std::net::{AddrParseError, SocketAddr};

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream, future};
use futures::stream::FuturesUnordered;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::{Core, Handle};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;
use tokio_service::{NewService, Service};

use super::line_codec::LineCodec;
use super::line_protocol::LineProtocol;
use super::mock_line_service;
use super::mock_line_service::{HandleRequest, MockLineService,
                               MockLineServiceFactory};

error_chain! {
    links {
        ServiceError(mock_line_service::Error, mock_line_service::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        InvalidAddressToBindTo(AddrParseError);
    }

    errors {
        FailedToReceiveConnection {
            description("failed to receive a connection")
        }

        ActiveStatusHasNoPollEquivalent {
            description("active server status means processing hasn't finished")
        }
    }
}

pub struct MockLineServer {
    address: SocketAddr,
    service_factory: MockLineServiceFactory,
}

pub type ServerFuture = Box<Future<Item = (), Error = Error>>;

impl MockLineServer {
    pub fn new(address: SocketAddr) -> MockLineServer {
        Self {
            address,
            service_factory: MockLineServiceFactory::new(),
        }
    }

    pub fn expect(&mut self, request: &str, response: &str) -> &mut Self {
        self.service_factory.expect(request, response);

        self
    }

    pub fn serve(&mut self) -> Result<()> {
        match Core::new() {
            Ok(mut reactor) => {
                let server = self.serve_with_handle(reactor.handle());
                reactor.run(server)
            },
            Err(error) => Err(error.into())
        }
    }

    pub fn serve_with_handle(&mut self, handle: Handle) -> ServerFuture {
        match TcpListener::bind(&self.address, &handle) {
            Ok(listener) => self.serve_on_listener(listener),
            Err(error) => future::result(Err(error.into())).boxed()
        }
    }

    fn serve_on_listener(&mut self, listener: TcpListener) -> ServerFuture {
        let connections = listener.incoming();
        let single_connection = connections.take(1)
            .into_future()
            .map(|(maybe_connection, _)| {
                let no_connections = ErrorKind::FailedToReceiveConnection;
                let no_connections: Error = no_connections.into();

                future::result(maybe_connection.ok_or(no_connections))
            })
            .map_err::<_, Error>(|(error, _)| error.into())
            .flatten();

        let service = self.service_factory.new_service();
        let server = single_connection.map(|(socket, client_address)| {
            let protocol = LineProtocol::with_separator('\n' as u8);
            let protocol_stream = protocol.bind_transport(socket);

            future::result(protocol_stream)
                .join(future::result(service))
                .map_err::<_, Error>(|error| error.into())
                .map(|(connection, service)| {
                    ActiveMockLineServer::new(connection, service)
                })
                .flatten()
        });

        server.flatten().boxed()
    }
}

#[derive(Debug)]
enum Status {
    Active,
    Finished,
    WouldBlock,
    Error(Error),
}

impl Status {
    fn is_active(&self) -> bool {
        match *self {
            Status::Active => true,
            _ => false
        }
    }

    fn is_more_severe_than(&self, other: &Status) -> bool {
        match (self, other) {
            (_, &Status::Error(_)) => false,
            (&Status::Error(_), _) => true,
            (_, &Status::WouldBlock) => false,
            (&Status::WouldBlock, _) => true,
            (_, &Status::Finished) => false,
            _ => true
        }
    }

    fn update<T: Into<Status>>(&mut self, status_update: T) {
        let status_update = status_update.into();

        if status_update.is_more_severe_than(self) {
            *self = status_update;
        }
    }
}

impl<T, E> From<Poll<T, E>> for Status
where E: Into<Error> {
    fn from(poll: Poll<T, E>) -> Status {
        match poll {
            Ok(Async::Ready(_)) => Status::Active,
            Ok(Async::NotReady) => Status::WouldBlock,
            Err(error) => Status::Error(error.into()),
        }
    }
}

impl<T, E> From<StartSend<T, E>> for Status
where E: Into<Error> {
    fn from(start_send: StartSend<T, E>) -> Status {
        match start_send {
            Ok(AsyncSink::Ready) => Status::Active,
            Ok(AsyncSink::NotReady(_)) => Status::WouldBlock,
            Err(error) => Status::Error(error.into()),
        }
    }
}

impl Into<Poll<(), Error>> for Status {
    fn into(self) -> Poll<(), Error> {
        match self {
            Status::Finished => Ok(Async::Ready(())),
            Status::WouldBlock => Ok(Async::NotReady),
            Status::Error(error) => Err(error),
            Status::Active =>
                Err(ErrorKind::ActiveStatusHasNoPollEquivalent.into()),
        }
    }
}

pub struct ActiveMockLineServer {
    connection: Framed<TcpStream, LineCodec>,
    service: MockLineService,
    live_requests: FuturesUnordered<HandleRequest>,
    live_responses: Vec<String>,
    status: Status,
}

impl ActiveMockLineServer {
    fn new(connection: Framed<TcpStream, LineCodec>, service: MockLineService)
        -> Self
    {
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

    fn send_responses_while_possible(&mut self)
        -> Option<(usize, StartSend<String, io::Error>)>
    {
        let connection = &mut self.connection;

        self.live_responses.iter()
            .map(|response| connection.start_send(response.clone()))
            .enumerate()
            .find(|&(_, ref status)| match *status {
                Ok(AsyncSink::Ready) => false,
                Ok(AsyncSink::NotReady(_)) => true,
                Err(_) => true
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
