use std::fmt::Display;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{Future, IntoFuture};
use futures::future::Flatten;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};
use tokio_proto::pipeline::ServerProto;

use super::errors::Result;
use super::mock_server_start::MockServerStart;
use super::super::mock_service::{MockServiceFactory, When};

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
    P::Request: Clone + Display + Eq + Hash + Send,
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

    pub fn when<A>(&mut self, request: A) -> When<P::Request, P::Response>
    where
        A: Into<P::Request>,
    {
        self.service_factory.when(request.into())
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

    pub fn serve_with_handle(
        &mut self,
        handle: Handle,
    ) -> Flatten<MockServerStart<P>> {
        self.start(handle).flatten()
    }

    pub fn start(&mut self, handle: Handle) -> MockServerStart<P> {
        let address = self.address.clone();
        let protocol = self.protocol.clone();
        let service_factory = self.service_factory.clone();

        MockServerStart::new(address, service_factory, protocol, handle)
    }
}
