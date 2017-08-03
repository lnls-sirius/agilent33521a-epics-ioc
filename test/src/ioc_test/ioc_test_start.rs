use std::fmt::Display;
use std::hash::Hash;

use futures::{Async, Future, Poll};
use tokio_core::net::TcpStream;
use tokio_proto::pipeline::ServerProto;

use super::errors::Error;
use super::ioc_test_start_ioc::IocTestStartIoc;
use super::super::ioc::IocSpawn;
use super::super::mock_server::MockServerStart;

pub struct IocTestStart<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
{
    ioc: Option<IocSpawn>,
    server: MockServerStart<P>,
    ioc_variables_to_set: Option<Vec<(String, String)>>,
}

impl<P> IocTestStart<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
{
    pub fn new(
        ioc: IocSpawn,
        server: MockServerStart<P>,
        ioc_variables_to_set: Vec<(String, String)>,
    ) -> Self {
        Self {
            server,
            ioc: Some(ioc),
            ioc_variables_to_set: Some(ioc_variables_to_set),
        }
    }

    fn take_parameters_to_forward(
        &mut self,
    ) -> (IocSpawn, Vec<(String, String)>) {
        let error_message = "IocTestStart polled after it finished";

        let ioc = self.ioc.take().expect(error_message);
        let ioc_variables_to_set =
            self.ioc_variables_to_set.take().expect(error_message);

        (ioc, ioc_variables_to_set)
    }
}

impl<P> Future for IocTestStart<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
{
    type Item = IocTestStartIoc<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let listening_server = try_ready!(self.server.poll());

        let (ioc, ioc_variables_to_set) = self.take_parameters_to_forward();

        Ok(Async::Ready(IocTestStartIoc::new(
            ioc,
            listening_server,
            ioc_variables_to_set,
        )))
    }
}
