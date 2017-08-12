use ioc_test::tests::Protocol;
use ioc_test::ioc_test::IocTestSetup;
use ioc_test::scpi::ScpiRequest;
use ioc_test::scpi::ScpiResponse;
use ioc_test::test_scheduler::TestScheduler;
use ioc_test::test_spawner::TestSpawner;

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
