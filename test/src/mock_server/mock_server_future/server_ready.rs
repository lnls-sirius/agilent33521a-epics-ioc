use std::fmt::Display;

use tokio_core::net::TcpStream;
use tokio_proto::pipeline::ServerProto;

use futures::{Future, Poll};
use super::state::State;
use super::super::active_mock_server::ActiveMockServer;
use super::super::errors::Error;
use super::super::super::mock_service::MockService;

pub struct ServerReady<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    server: ActiveMockServer<P::Transport>,
}

impl<P> ServerReady<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    pub fn advance_with(
        parameters_tuple: (P::Transport, MockService<P::Request, P::Response>),
    ) -> (Poll<(), Error>, State<P>) {
        let server_ready = Self {
            server: ActiveMockServer::from_tuple(parameters_tuple),
        };

        server_ready.advance()
    }

    pub fn advance(mut self) -> (Poll<(), Error>, State<P>) {
        (self.server.poll(), State::ServerReady(self))
    }
}
