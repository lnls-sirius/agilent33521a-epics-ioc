use std::net::SocketAddr;

use futures::{future, Future, Stream};
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tokio_proto::pipeline::ServerProto;
use tokio_service::NewService;

use super::active_mock_line_server::ActiveMockLineServer;
use super::errors::{Error, ErrorKind, Result};
use super::super::line_protocol::LineProtocol;
use super::super::mock_line_service::MockLineServiceFactory;

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
        let server = single_connection.map(|(socket, _client_address)| {
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

