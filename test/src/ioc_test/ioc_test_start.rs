use futures::{Async, Future, Poll};

use super::errors::Error;
use super::ioc_test::IocTest;
use super::super::ioc::IocInstance;
use super::super::ioc::IocSpawn;
use super::super::line_protocol::LineProtocol;
use super::super::mock_server::MockServerStart;

pub struct IocTestStart {
    ioc: IocSpawn,
    server: MockServerStart<LineProtocol>,
    ioc_variables_to_set: Vec<(String, String)>,
}

impl IocTestStart {
    pub fn new(
        ioc: IocSpawn,
        server: MockServerStart<LineProtocol>,
        ioc_variables_to_set: Vec<(String, String)>,
    ) -> Self {
        Self {
            ioc,
            server,
            ioc_variables_to_set,
        }
    }
}

impl Future for IocTestStart {
    type Item = IocTest;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let listening_server = try_ready!(self.server.poll());

        let ioc_process = match self.ioc.poll() {
            Ok(Async::Ready(process)) => process,
            _ => panic!("IocSpawn was expected to spawn the IOC immediately"),
        };
        let mut ioc_instance = IocInstance::new(ioc_process);

        for &(ref name, ref value) in self.ioc_variables_to_set.iter() {
            ioc_instance.set_variable(name, value);
        }

        Ok(Async::Ready(
            IocTest::new(ioc_instance, listening_server.flatten()),
        ))
    }
}
