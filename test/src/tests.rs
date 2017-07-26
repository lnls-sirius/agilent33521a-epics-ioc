use std::io;

use futures::IntoFuture;
use tokio_core::reactor::Core;

use super::ioc_test;
use super::ioc_test_setup;
use super::ioc_test_setup::IocTestSetup;

error_chain! {
    links {
        SetupError(ioc_test_setup::Error, ioc_test_setup::ErrorKind);
        TestError(ioc_test::Error, ioc_test::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
    }
}

fn test_enable_channel_output(test: &mut IocTestSetup) {
    test.when("OUTPut1 ON").reply_with("");

    test.set_variable("channelOutput-Sel", "ON");
}

pub fn run_test() -> Result<()> {
    let mut reactor = Core::new()?;
    let port = 55000;

    let mut test = IocTestSetup::new(reactor.handle(), port)?;

    test_enable_channel_output(&mut test);

    reactor.run(test.into_future())?;

    Ok(())
}
