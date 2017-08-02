use std::hash::Hash;

use super::mock_service_factory::MockServiceFactory;

pub struct When<'a, A, B>
where
    A: Clone + Eq + Hash + 'a,
    B: Clone + 'a,
{
    parent: &'a mut MockServiceFactory<A, B>,
    request: A,
}

impl<'a, A, B> When<'a, A, B>
where
    A: Clone + Eq + Hash,
    B: Clone,
{
    pub fn new(parent: &'a mut MockServiceFactory<A, B>, request: A) -> Self {
        Self { parent, request }
    }

    pub fn reply_with<C>(self, response: C) -> Self
    where
        C: Into<B>,
    {
        self.parent.expect(self.request.clone(), response.into());

        self
    }

    pub fn when<C>(self, request: C) -> Self
    where
        C: Into<A>,
    {
        Self::new(self.parent, request.into())
    }
}
