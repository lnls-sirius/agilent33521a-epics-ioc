use std::hash::Hash;

use super::mock_service_factory::MockServiceFactory;

pub struct When<'a, A, B>
where
    A: Eq + Hash + 'a,
    B: 'a,
{
    parent: &'a mut MockServiceFactory<A, B>,
    request: A,
}

impl<'a, A, B> When<'a, A, B>
where
    A: Clone + Eq + Hash,
{
    pub fn new(parent: &'a mut MockServiceFactory<A, B>, request: A) -> Self {
        Self { parent, request }
    }

    pub fn reply_with<D>(self, response: D) -> Self
    where
        D: Into<B>,
    {
        self.parent.expect(self.request.clone(), response.into());

        self
    }

    pub fn verify(self) -> Self {
        self.parent.verify(self.request.clone());

        self
    }

    pub fn when<C>(self, request: C) -> Self
    where
        C: Into<A>,
    {
        Self::new(self.parent, request.into())
    }
}
