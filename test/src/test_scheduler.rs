use std::mem;

use futures::{Async, Future, IntoFuture, Poll};
use super::test_result::TestResult;

pub struct TestScheduler<S, T, E>
where
    S: FnMut() -> T,
    T: IntoFuture<Item = TestResult<E>, Error = ()>,
{
    tests: Vec<Box<FnMut(&mut T)>>,
    spawner: Option<S>,
    test_executions: Vec<T::Future>,
    test_results: Vec<TestResult<E>>,
}

impl<S, T, E> TestScheduler<S, T, E>
where
    S: FnMut() -> T,
    T: IntoFuture<Item = TestResult<E>, Error = ()>,
{
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            spawner: None,
            test_executions: Vec::new(),
            test_results: Vec::new(),
        }
    }

    pub fn spawn(&mut self, spawner: S) {
        self.spawner = Some(spawner);
    }

    pub fn add<F>(&mut self, test_setup: F)
    where
        F: FnMut(&mut T) + 'static,
    {
        self.tests.push(Box::new(test_setup));
    }
}

impl<S, T, E> Future for TestScheduler<S, T, E>
where
    S: FnMut() -> T,
    T: IntoFuture<Item = TestResult<E>, Error = ()>,
{
    type Item = Vec<TestResult<E>>;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut poll_status = Err(());

        if let Some(ref mut spawner) = self.spawner {
            for mut test_setup_function in self.tests.drain(0..) {
                let mut test = spawner();

                test_setup_function(&mut test);

                self.test_executions.push(test.into_future());
            }

            let test_poll_results: Vec<Poll<_, _>> = self.test_executions
                .iter_mut()
                .map(|execution| execution.poll())
                .collect();

            for (poll_result, index) in test_poll_results.into_iter().zip(0..) {
                match poll_result {
                    Ok(Async::Ready(result)) => {
                        self.test_results.push(result);
                        self.test_executions.remove(index);
                    }
                    Ok(Async::NotReady) => {}
                    Err(_) => panic!("Fatal test execution failure"),
                }
            }

            if self.tests.is_empty() && self.test_executions.is_empty() {
                let test_results =
                    mem::replace(&mut self.test_results, Vec::new());

                poll_status = Ok(Async::Ready(test_results));
            } else {
                poll_status = Ok(Async::NotReady);
            }
        }

        poll_status
    }
}
