use crate::growth_count::GrowthCount;

pub struct HandleSwitch<'a> {
    growth_count: &'a GrowthCount,
}

impl<'a> HandleSwitch<'a> {
    pub fn new(growth_count: &'a GrowthCount) -> Self {
        Self { growth_count }
    }
}

impl Drop for HandleSwitch<'_> {
    fn drop(&mut self) {
        self.growth_count.switched();
    }
}
