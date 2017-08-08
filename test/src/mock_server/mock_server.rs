use std::fmt::Display;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::Future;
use futures::future::Flatten;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};
use tokio_proto::pipeline::ServerProto;

use super::errors::{Error, Result};
use super::mock_server_start::MockServerStart;
use super::super::mock_service::{MockServiceFactory, When};

pub struct MockServer<P>
where
    P: ServerProto<TcpStream>,
{
    address: SocketAddr,
    service_factory: MockServiceFactory<P::Request, P::Response>,
    protocol: Arc<Mutex<P>>,
}

impl<P> MockServer<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
    P::Error: Into<Error>,
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

    pub fn verify<A>(&mut self, request: A)
    where
        A: Into<P::Request>,
    {
        self.service_factory.verify(request);
    }

    pub fn serve(self) -> Result<()> {
        match Core::new() {
            Ok(mut reactor) => {
                let server = self.serve_with_handle(reactor.handle());
                reactor.run(server)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn serve_with_handle(
        self,
        handle: Handle,
    ) -> Flatten<Flatten<MockServerStart<P>>> {
        self.start(handle).flatten().flatten()
    }

    pub fn start(self, handle: Handle) -> MockServerStart<P> {
        let address = self.address;
        let protocol = self.protocol;
        let service_factory = self.service_factory;

        MockServerStart::new(address, service_factory, protocol, handle)
    }
}
