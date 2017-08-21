#[macro_use]
extern crate ioc_test;
extern crate tokio_core;

mod output;

use std::io;

use ioc_test::{IocTestParameters, IocTestSetup, IocTestSpawner,
               MockTestParameters, TestReporter, TestScheduler};
use ioc_test::scpi::{ScpiProtocol, ScpiRequest, ScpiResponse};
use tokio_core::reactor::Core;

pub fn run_tests() -> Result<(), io::Error> {
    let mut reactor = Core::new()?;
    let ioc_command = "/project/iocBoot/iocagilent33521a/run.sh";

    let spawner = IocTestSpawner::new(
        ioc_command,
        reactor.handle(),
        configure_initial_test_messages,
        MockTestParameters::new(ScpiProtocol),
    );

    let mut tests = TestScheduler::new(spawner);

    output::add_tests(&mut tests);

    Ok(reactor.run(TestReporter::new(tests)).unwrap())
}

fn configure_initial_test_messages<P>(test: &mut IocTestSetup<P>)
where
    P: IocTestParameters,
    P::Request: From<ScpiRequest>,
    P::Response: From<ScpiResponse>,
{
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
