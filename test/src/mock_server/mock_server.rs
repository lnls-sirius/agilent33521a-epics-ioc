use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{future, Future, IntoFuture};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::{Core, Handle};
use tokio_proto::pipeline::ServerProto;
use tokio_service::NewService;

use super::active_mock_server::ActiveMockServer;
use super::bound_connection_future::BoundConnectionFuture;
use super::errors::{Error, NormalizeError, Result};
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
        let service = self.service_factory.new_service();
        let protocol = self.protocol.clone();
        let connection = BoundConnectionFuture::from(listener, protocol);

        let server = connection
            .join(service.normalize_error())
            .map(ActiveMockServer::from_tuple)
            .flatten();

        server.boxed()
    }
}
