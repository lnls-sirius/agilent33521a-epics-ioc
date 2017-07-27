use futures::{Async, Future, Poll};

use super::errors::Error;
use super::ioc_test_start_ioc::IocTestStartIoc;
use super::super::ioc::IocSpawn;
use super::super::line_protocol::LineProtocol;
use super::super::mock_server::MockServerStart;

pub struct IocTestStart {
    ioc: Option<IocSpawn>,
    server: MockServerStart<LineProtocol>,
    ioc_variables_to_set: Option<Vec<(String, String)>>,
}

impl IocTestStart {
    pub fn new(
        ioc: IocSpawn,
        server: MockServerStart<LineProtocol>,
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

impl Future for IocTestStart {
    type Item = IocTestStartIoc;
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
