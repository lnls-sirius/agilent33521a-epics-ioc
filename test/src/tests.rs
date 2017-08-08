use std::io;

use futures::{Future, IntoFuture};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_proto::pipeline::ServerProto;

use super::line_protocol::LineProtocol;
use super::ioc_test::{IocTestSetup, Result};

trait Protocol:
    ServerProto<
        TcpStream,
        Request = String,
        Response = String,
        Error = io::Error>
{
}

impl Protocol for LineProtocol {}

fn test_enable_channel_output<P: Protocol>(test: &mut IocTestSetup<P>) {
    test.when("OUTPut1 ON").reply_with("");

    test.set_variable("channelOutput-Sel", "ON");

    test.verify("OUTPut1 ON");
}

pub fn run_test() -> Result<()> {
    let mut reactor = Core::new()?;
    let protocol = LineProtocol::with_separator('\n' as u8);
    let port = 55000;

    let mut test = IocTestSetup::new(reactor.handle(), protocol, port)?;

    test_enable_channel_output(&mut test);

    reactor.run(test.into_future().flatten().flatten())?;

    Ok(())
}
