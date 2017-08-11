pub struct TestResult<E> {
    test: String,
    result: Result<(), E>,
}

impl<E> TestResult<E> {
    pub fn success(test: String) -> Self {
        Self {
            test,
            result: Ok(()),
        }
    }

    pub fn failure(test: String, error: E) -> Self {
        Self {
            test,
            result: Err(error),
        }
    }

    pub fn name(&self) -> &str {
        self.test.as_str()
    }

    pub fn result(&self) -> &Result<(), E> {
        &self.result
    }
}
