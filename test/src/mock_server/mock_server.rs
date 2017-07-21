use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::IntoFuture;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};
use tokio_proto::pipeline::ServerProto;

use super::errors::Result;
use super::mock_server_future::MockServerFuture;
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

    pub fn serve_with_handle(&mut self, handle: Handle) -> MockServerFuture<P> {
        let address = self.address.clone();
        let protocol = self.protocol.clone();
        let service_factory = self.service_factory.clone();

        MockServerFuture::new(address, service_factory, protocol, handle)
    }
}
