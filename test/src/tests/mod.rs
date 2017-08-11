#[macro_use]
mod setup;

mod output;

use self::setup::Protocol;
use super::ioc_test::{Error, IocTestSetup};
use super::test_scheduler::TestScheduler;

pub use self::setup::run_tests;

pub fn add_tests<S, P>(scheduler: &mut TestScheduler<S, IocTestSetup<P>, Error>)
where
    S: FnMut() -> IocTestSetup<P>,
    P: Protocol,
{
    output::add_tests(scheduler);
}
