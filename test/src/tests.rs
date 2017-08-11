use futures::IntoFuture;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_proto::pipeline::ServerProto;

use super::ioc_test::{IocTestSetup, Result};
use super::scpi;
use super::scpi::ScpiProtocol;
use super::scpi::ScpiRequest;
use super::scpi::ScpiResponse;

trait Protocol
    : ServerProto<
    TcpStream,
    Request = ScpiRequest,
    Response = ScpiResponse,
    Error = scpi::Error,
> {
}

impl Protocol for ScpiProtocol {}

fn test_enable_channel_output<P: Protocol>(test: &mut IocTestSetup<P>) {
    let output1_on = ScpiRequest::OutputOn(1);

    test.when(output1_on.clone())
        .reply_with(ScpiResponse::Empty);

    test.set_variable("channelOutput-Sel", "ON");

    test.verify(output1_on);
}

pub fn run_test() -> Result<()> {
    let mut reactor = Core::new()?;
    let protocol = ScpiProtocol;
    let port = 55000;

    let mut test = IocTestSetup::new(reactor.handle(), protocol, port)?;

    configure_initial_test_messages(&mut test);
    test_enable_channel_output(&mut test);

    reactor.run(test.into_future())?;

    Ok(())
}

fn configure_initial_test_messages<P: Protocol>(test: &mut IocTestSetup<P>) {
    request_response_map! { test,
        ScpiRequest::OutputStatus(1) => ScpiResponse::Integer(0),
        ScpiRequest::SourceFrequencyGet(1) => ScpiResponse::Integer(1),
        ScpiRequest::SourcePhaseGet(1) => ScpiResponse::Integer(1),
        ScpiRequest::SourceVoltageGet(1) => ScpiResponse::Integer(1),
        ScpiRequest::SourceVoltageOffsetGet(1) => ScpiResponse::Integer(1),
        ScpiRequest::SourceFunctionQuery(1) =>
            ScpiResponse::Utf8String(String::from("SQUare")),
        ScpiRequest::SourceArbitraryFunctionFileQuery(1) =>
            ScpiResponse::Utf8String(String::from("\"DUMMY.FILE\"")),
        ScpiRequest::SourceArbitraryFunctionSampleRateGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourceNoiseFunctionBandwidthGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourcePrbsFunctionBitRateGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourcePrbsFunctionPolynomialGet(1) =>
            ScpiResponse::Utf8String(String::from("PN7")),
        ScpiRequest::SourcePrbsFunctionTransitionGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourcePulseFunctionLeadingEdgeTransitionGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourcePulseFunctionTrailingEdgeTransitionGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourcePulseFunctionPulseWidthGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourceRampFunctionSymmetryGet(1) =>
            ScpiResponse::Integer(1),
        ScpiRequest::SourceSquareFunctionDutyCycleGet(1) =>
            ScpiResponse::Integer(1),
    };
}
