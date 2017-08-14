use ioc_test::scpi::{ScpiRequest, ScpiResponse};
use ioc_test::{IocTestProtocol, IocTestSetup, TestScheduler, TestSpawner};

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
