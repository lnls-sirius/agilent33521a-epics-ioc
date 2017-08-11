use super::setup::Protocol;
use super::super::ioc_test::{Error, IocTestSetup};
use super::super::scpi::ScpiRequest;
use super::super::scpi::ScpiResponse;
use super::super::test_scheduler::TestScheduler;

tests! {
    test("enable channel output") {
        test.set_variable("channelOutput-Sel", "ON");

        test.when(ScpiRequest::OutputOn(1))
            .reply_with(ScpiResponse::Empty)
            .verify();
    }

    test("disable channel output") {
        test.set_variable("channelOutput-Sel", "OFF");

        test.when(ScpiRequest::OutputOff(1))
            .reply_with(ScpiResponse::Empty)
            .verify();
    }
}
