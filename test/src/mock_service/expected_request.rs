#[derive(Clone, Default)]
pub struct ExpectedRequest<A, B>
where
    A: Clone,
    B: Clone,
{
    pub request: A,
    pub response: B,
}
