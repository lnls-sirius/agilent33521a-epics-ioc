use std::fmt::Display;
use std::hash::Hash;

use futures::{Future, IntoFuture};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_proto::pipeline::ServerProto;

use super::line_protocol::LineProtocol;
use super::ioc_test::{IocTestSetup, Result};

fn test_enable_channel_output<'a, 'b, P>(test: &mut IocTestSetup<P>)
where
    P: ServerProto<TcpStream> + Send,
    <P as ServerProto<TcpStream>>::Request:
        Clone + Display + Eq + From<&'a str> + Hash + Send,
    <P as ServerProto<TcpStream>>::Response: Clone + From<&'b str> + Send,
    <P as ServerProto<TcpStream>>::Transport: Send,
    <<P as ServerProto<TcpStream>>::BindTransport as IntoFuture>::Future: Send,
{
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
