use std::fmt::Display;
use std::hash::Hash;

use futures::{Async, Future, Poll};
use tokio_core::net::TcpStream;
use tokio_proto::pipeline::ServerProto;

use super::errors::Error;
use super::ioc_test_execution::IocTestExecution;
use super::super::ioc::IocInstance;
use super::super::ioc::IocProcess;
use super::super::ioc::IocSpawn;
use super::super::mock_server;
use super::super::mock_server::ListeningMockServer;

pub struct IocTestStartIoc<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    ioc: IocSpawn,
    listening_server: Option<ListeningMockServer<P>>,
    ioc_variables_to_set: Vec<(String, String)>,
}

impl<P> IocTestStartIoc<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    pub fn new(
        ioc: IocSpawn,
        listening_server: ListeningMockServer<P>,
        ioc_variables_to_set: Vec<(String, String)>,
    ) -> Self {
        Self {
            ioc,
            ioc_variables_to_set,
            listening_server: Some(listening_server),
        }
    }
}

impl<P> Future for IocTestStartIoc<P>
where
    P: ServerProto<TcpStream>,
    <P as ServerProto<TcpStream>>::Request: Clone + Display + Eq + Hash,
    <P as ServerProto<TcpStream>>::Response: Clone,
    <P as ServerProto<TcpStream>>::Error: Into<mock_server::Error>,
{
    type Item = IocTestExecution<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let ioc_process = try_ready!(self.ioc.poll());
        let ioc_process = IocProcess::new(ioc_process)?;
        let mut ioc = IocInstance::new(ioc_process);

        for &(ref name, ref value) in self.ioc_variables_to_set.iter() {
            ioc.set_variable(name, value);
        }

        let listening_server = self.listening_server
            .take()
            .expect("IocTestStartIoc polled after it finished");
        let server = listening_server.flatten();

        Ok(Async::Ready(IocTestExecution::new(ioc, server)))
    }
}
