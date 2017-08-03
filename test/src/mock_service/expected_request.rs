pub struct ExpectedRequest<A, B> {
    pub request: A,
    pub response: B,
}

impl<A, B> Clone for ExpectedRequest<A, B>
where
    A: Clone,
    B: Clone,
{
    fn clone(&self) -> ExpectedRequest<A, B> {
        ExpectedRequest {
            request: self.request.clone(),
            response: self.response.clone(),
        }
    }
}
