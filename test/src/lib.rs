#[macro_use]
extern crate ioc_test;
extern crate tokio_core;

mod output;

use std::io;

use ioc_test::{IocTestSpawner, TestReporter, TestScheduler};
use tokio_core::reactor::Core;

pub fn run_tests() -> Result<(), io::Error> {
    let mut reactor = Core::new()?;
    let ioc_command = "/project/iocBoot/iocagilent33521a/run.sh";
    let spawner = IocTestSpawner::new(ioc_command, reactor.handle());
    let mut tests = TestScheduler::new(spawner);

    output::add_tests(&mut tests);

    Ok(reactor.run(TestReporter::new(tests)).unwrap())
}
