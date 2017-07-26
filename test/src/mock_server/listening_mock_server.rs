use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use futures::{Async, Future, Poll};
use futures::future::{FutureResult, Join};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_proto::pipeline::ServerProto;
use tokio_service::NewService;

use super::active_mock_server::ActiveMockServer;
use super::bound_connection_future::BoundConnectionFuture;
use super::errors::{Error, NormalizeError};
use super::super::mock_service::MockService;
use super::super::mock_service::MockServiceFactory;

pub struct ListeningMockServer<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    connection_and_service: Join<
        BoundConnectionFuture<P>,
        FutureResult<MockService<P::Request, P::Response>, Error>,
    >,
}

impl<P> ListeningMockServer<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    pub fn new(
        listener: TcpListener,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
    ) -> Self {
        let service = service_factory.new_service();
        let connection = BoundConnectionFuture::from(listener, protocol);

        Self {
            connection_and_service: connection.join(service.normalize_error()),
        }
    }
}

impl<P> Future for ListeningMockServer<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    type Item = ActiveMockServer<P::Transport>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (connection, service) =
            try_ready!(self.connection_and_service.poll());

        Ok(Async::Ready(ActiveMockServer::new(connection, service)))
    }
}
