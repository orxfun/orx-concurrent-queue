use crate::state::State;

pub struct HandleGrowth<'a> {
    growth_count: &'a State,
}

impl<'a> HandleGrowth<'a> {
    pub fn new(growth_count: &'a State) -> Self {
        Self { growth_count }
    }
}

impl Drop for HandleGrowth<'_> {
    fn drop(&mut self) {
        self.growth_count.grew_once();
    }
}
