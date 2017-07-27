use futures::{Async, Future, Poll};

use super::errors::Error;
use super::ioc_test::IocTest;
use super::super::ioc::IocInstance;
use super::super::ioc::IocProcess;
use super::super::ioc::IocSpawn;
use super::super::line_protocol::LineProtocol;
use super::super::mock_server::ListeningMockServer;

pub struct IocTestStartIoc {
    ioc: IocSpawn,
    listening_server: Option<ListeningMockServer<LineProtocol>>,
    ioc_variables_to_set: Vec<(String, String)>,
}

impl IocTestStartIoc {
    pub fn new(
        ioc: IocSpawn,
        listening_server: ListeningMockServer<LineProtocol>,
        ioc_variables_to_set: Vec<(String, String)>,
    ) -> Self {
        Self {
            ioc,
            ioc_variables_to_set,
            listening_server: Some(listening_server),
        }
    }
}

impl Future for IocTestStartIoc {
    type Item = IocTest;
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

        Ok(Async::Ready(IocTest::new(ioc, server)))
    }
}
