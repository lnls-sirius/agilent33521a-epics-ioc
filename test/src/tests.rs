use futures::IntoFuture;
use tokio_core::reactor::Core;

use super::ioc_test::{IocTestSetup, Result};

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
