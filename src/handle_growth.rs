use crate::growth_count::GrowthCount;

pub struct HandleGrowth<'a> {
    growth_count: &'a GrowthCount,
}

impl<'a> HandleGrowth<'a> {
    pub fn new(growth_count: &'a GrowthCount) -> Self {
        Self { growth_count }
    }
}

impl Drop for HandleGrowth<'_> {
    fn drop(&mut self) {
        self.growth_count.grew_once();
    }
}
