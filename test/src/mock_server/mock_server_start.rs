use std::fmt::Display;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{Async, Future, Poll};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ServerProto;

use super::errors::{Error, ErrorKind};
use super::listening_mock_server::ListeningMockServer;
use super::super::mock_service::MockServiceFactory;

pub struct MockServerStart<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    address: SocketAddr,
    service_factory: Option<MockServiceFactory<P::Request, P::Response>>,
    protocol: Arc<Mutex<P>>,
    handle: Handle,
}

impl<P> MockServerStart<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    pub fn new(
        address: SocketAddr,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
        handle: Handle,
    ) -> Self {
        Self {
            address,
            protocol,
            handle,
            service_factory: Some(service_factory),
        }
    }
}

impl<P> Future for MockServerStart<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    type Item = ListeningMockServer<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.service_factory.is_some() {
            let listener = TcpListener::bind(&self.address, &self.handle)?;
            let protocol = self.protocol.clone();

            if let Some(service_factory) = self.service_factory.take() {
                Ok(Async::Ready(ListeningMockServer::new(
                    listener,
                    service_factory,
                    protocol,
                )))
            } else {
                Err(ErrorKind::AttemptToStartServerTwice.into())
            }
        } else {
            Err(ErrorKind::AttemptToStartServerTwice.into())
        }
    }
}
