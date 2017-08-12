#[macro_use]
extern crate ioc_test;
extern crate tokio_core;

mod output;

use std::io;

use ioc_test::tests::IocTestSpawner;
use ioc_test::test_reporter::TestReporter;
use ioc_test::test_scheduler::TestScheduler;
use tokio_core::reactor::Core;

pub fn run_tests() -> Result<(), io::Error> {
    let mut reactor = Core::new()?;
    let spawner = IocTestSpawner::new(reactor.handle());
    let mut tests = TestScheduler::new(spawner);

    output::add_tests(&mut tests);

    Ok(reactor.run(TestReporter::new(tests)).unwrap())
}
