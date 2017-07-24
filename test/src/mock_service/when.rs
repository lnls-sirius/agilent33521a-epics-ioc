use super::mock_service_factory::MockServiceFactory;

pub struct When<'a, A, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
{
    parent: &'a mut MockServiceFactory<A, B>,
    request: A,
}

impl<'a, A, B> When<'a, A, B>
where
    A: Clone,
    B: Clone,
{
    pub fn new(parent: &'a mut MockServiceFactory<A, B>, request: A) -> Self {
        Self { parent, request }
    }

    pub fn reply_with<C>(self, response: C)
    where
        C: Into<B>,
    {
        self.parent.expect(self.request, response.into());
    }
}
