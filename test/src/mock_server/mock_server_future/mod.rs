mod state;

use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{Future, Poll};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ServerProto;

use self::state::State;
use super::errors::Error;
use super::super::mock_service::MockServiceFactory;

pub struct MockServerFuture<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    state: State<P>,
}

impl<P> MockServerFuture<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    pub fn new(
        address: SocketAddr,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
        handle: Handle,
    ) -> MockServerFuture<P> {
        let state =
            State::start_with(address, service_factory, protocol, handle);

        Self { state }
    }
}

impl<P> Future for MockServerFuture<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.state.advance()
    }
}
