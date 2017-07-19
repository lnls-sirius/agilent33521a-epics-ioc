use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{future, Future, IntoFuture, Stream};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::{Core, Handle};
use tokio_proto::pipeline::ServerProto;
use tokio_service::NewService;

use super::active_mock_server::ActiveMockServer;
use super::errors::{Error, ErrorKind, Result};
use super::super::mock_service::MockServiceFactory;

pub struct MockServer<P>
where
    P: ServerProto<TcpStream> + Send,
    P::Request: Clone + Display + PartialEq + Send,
    P::Response: Clone + Send,
    P::Transport: Send,
    <P::BindTransport as IntoFuture>::Future: Send,
{
    address: SocketAddr,
    service_factory: MockServiceFactory<P::Request, P::Response>,
    protocol: Arc<Mutex<P>>,
}

pub type ServerFuture = Box<Future<Item = (), Error = Error>>;

impl<P> MockServer<P>
where
    P: ServerProto<TcpStream> + Send,
    P::Request: Clone + Display + PartialEq + Send,
    P::Response: Clone + Send,
    P::Transport: Send,
    <P::BindTransport as IntoFuture>::Future: Send,
{
    pub fn new(address: SocketAddr, protocol: P) -> MockServer<P> {
        Self {
            address,
            service_factory: MockServiceFactory::new(),
            protocol: Arc::new(Mutex::new(protocol)),
        }
    }

    pub fn expect<A, B>(&mut self, request: A, response: B) -> &mut Self
    where
        A: Into<P::Request>,
        B: Into<P::Response>,
    {
        self.service_factory.expect(request.into(), response.into());

        self
    }

    pub fn serve(&mut self) -> Result<()> {
        match Core::new() {
            Ok(mut reactor) => {
                let server = self.serve_with_handle(reactor.handle());
                reactor.run(server)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn serve_with_handle(&mut self, handle: Handle) -> ServerFuture {
        match TcpListener::bind(&self.address, &handle) {
            Ok(listener) => self.serve_on_listener(listener),
            Err(error) => future::result(Err(error.into())).boxed(),
        }
    }

    fn serve_on_listener(&mut self, listener: TcpListener) -> ServerFuture {
        let connections = listener.incoming();
        let single_connection = connections
            .take(1)
            .into_future()
            .map(|(maybe_connection, _)| {
                let no_connections = ErrorKind::FailedToReceiveConnection;
                let no_connections: Error = no_connections.into();

                future::result(maybe_connection.ok_or(no_connections))
            })
            .map_err::<_, Error>(|(error, _)| error.into())
            .flatten();

        let protocol = self.protocol.clone();
        let service = self.service_factory.new_service();
        let server = single_connection.map(move |(socket, _client_address)| {
            let lock_error: Error = ErrorKind::FailedToBindConnection.into();

            let connection =
                protocol.lock().map_err(|_| lock_error).map(|protocol| {
                    protocol
                        .bind_transport(socket)
                        .into_future()
                        .map_err::<_, Error>(|error| error.into())
                });

            connection
                .into_future()
                .map_err::<_, Error>(|error| error.into())
                .flatten()
                .join(service.map_err(|error| error.into()))
                .map(|(connection, service)| {
                    ActiveMockServer::new(connection, service)
                })
                .flatten()
        });

        server.flatten().boxed()
    }
}
