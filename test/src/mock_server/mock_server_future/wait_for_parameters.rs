use std::fmt::Display;
use std::sync::{Arc, Mutex};

use futures::{Async, Future, Poll};
use futures::future::{FutureResult, Join};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_proto::pipeline::ServerProto;
use tokio_service::NewService;

use super::state::ServerReady;
use super::state::State;
use super::super::bound_connection_future::BoundConnectionFuture;
use super::super::errors::{Error, NormalizeError};
use super::super::super::mock_service::MockService;
use super::super::super::mock_service::MockServiceFactory;

pub struct WaitForParameters<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    parameters: Join<
        BoundConnectionFuture<P>,
        FutureResult<MockService<P::Request, P::Response>, Error>,
    >,
}

impl<P> WaitForParameters<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    pub fn advance_with(
        listener: TcpListener,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
    ) -> (Poll<(), Error>, State<P>) {
        let service = service_factory.new_service();
        let connection = BoundConnectionFuture::from(listener, protocol);
        let parameters = connection.join(service.normalize_error());

        let wait_for_parameters = WaitForParameters { parameters };

        wait_for_parameters.advance()
    }

    pub fn advance(mut self) -> (Poll<(), Error>, State<P>) {
        match self.parameters.poll() {
            Ok(Async::Ready(parameters)) => self.create_server(parameters),
            Ok(Async::NotReady) => (Ok(Async::NotReady), self.same_state()),
            Err(error) => (Err(error), self.same_state()),
        }
    }

    fn create_server(
        self,
        parameters_tuple: (P::Transport, MockService<P::Request, P::Response>),
    ) -> (Poll<(), Error>, State<P>) {
        ServerReady::advance_with(parameters_tuple)
    }

    fn same_state(self) -> State<P> {
        State::WaitingForParameters(self)
    }
}
