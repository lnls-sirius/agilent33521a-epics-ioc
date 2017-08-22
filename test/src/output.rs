use ioc_test::scpi::{NoScpiExtension, ScpiProtocol, ScpiRequest, ScpiResponse};
use ioc_test::{IocTestSetup, MockTestParameters, TestScheduler, TestSpawner};

tests! {
    type Protocol = ScpiProtocol<NoScpiExtension>;

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
