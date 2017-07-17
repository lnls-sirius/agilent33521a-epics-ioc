use std::io;
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
    }
}

pub struct MockLineServer {
    address: SocketAddr,
    service_factory: MockLineServiceFactory,
}

pub type ServerFuture = Box<Future<Item = ActiveMockLineServer, Error = Error>>;

impl MockLineServer {
    pub fn new(address: SocketAddr) -> MockLineServer {
        Self {
            address,
            service_factory: MockLineServiceFactory::new(),
        }
    }

    pub fn expect(&mut self, request: String, response: String) -> &mut Self {
        self.service_factory.expect(request, response);

        self
    }

    pub fn serve(&mut self) -> ServerFuture {
        match Core::new() {
            Ok(reactor) => self.serve_with_handle(reactor.handle()),
            Err(error) => future::err(error.into()).boxed()
        }
    }

    pub fn serve_with_handle(&mut self, handle: Handle) -> ServerFuture {
        match TcpListener::bind(&self.address, &handle) {
            Ok(listener) => self.serve_on_listener(listener),
            Err(error) => future::err(error.into()).boxed()
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
        let server = single_connection.map(|(socket, _)| {
            let protocol = LineProtocol::with_separator('\n' as u8);
            let protocol_stream = protocol.bind_transport(socket);

            future::result(protocol_stream)
                .join(future::result(service))
                .map_err(|error| error.into())
                .map(|(connection, service)| {
                    ActiveMockLineServer::new(connection, service)
                })
        });

        server.flatten().boxed()
    }
}

pub struct ActiveMockLineServer {
    connection: Framed<TcpStream, LineCodec>,
    service: MockLineService,
    live_requests: FuturesUnordered<HandleRequest>,
    live_responses: Vec<String>,
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
        }
    }

    fn try_to_get_new_request(&mut self) -> Result<()> {
        let new_request = self.connection.poll();

        if let Ok(Async::Ready(Some(request))) = new_request {
            self.live_requests.push(self.service.call(request));
            Ok(())
        } else {
            new_request.and(Ok(())).map_err(|error| error.into())
        }
    }

    fn try_to_get_new_response(&mut self) -> Result<()> {
        let maybe_response = self.live_requests.poll();

        if let Ok(Async::Ready(Some(response))) = maybe_response {
            self.live_responses.push(response);
            Ok(())
        } else {
            maybe_response.and(Ok(())).map_err(|error| error.into())
        }
    }

    fn try_to_send_responses(&mut self) -> Poll<(), Error> {
        let first_failed_send = self.send_responses_while_possible();

        if let Some((index, status)) = first_failed_send {
            self.live_responses.drain(0..index);

            match status {
                Ok(AsyncSink::Ready) => Ok(Async::Ready(())),
                Ok(AsyncSink::NotReady(_)) => Ok(Async::NotReady),
                Err(error) => Err(error.into())
            }
        } else {
            self.live_responses.clear();

            Ok(Async::Ready(()))
        }
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

    fn try_to_flush_responses(&mut self) -> Poll<(), Error> {
        self.connection.poll_complete().map_err(|error| error.into())
    }

    fn check_service_status(&mut self) -> Poll<(), Error> {
        self.service.poll().map_err(|error| error.into())
    }
}

impl Future for ActiveMockLineServer {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.try_to_get_new_request()
            .and_then(|_| self.try_to_get_new_response())
            .and_then(|_| self.try_to_send_responses())
            .and_then(|_| self.try_to_flush_responses())
            .and_then(|_| self.check_service_status())
            .and_then(|status| {
                let pending_requests = !self.live_requests.is_empty();
                let pending_responses = !self.live_responses.is_empty();

                if pending_requests || pending_responses {
                    Ok(Async::NotReady)
                } else {
                    Ok(status)
                }
            })
    }
}
