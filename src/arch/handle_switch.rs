use crate::state::State;

pub struct HandleSwitch<'a> {
    growth_count: &'a State,
}

impl<'a> HandleSwitch<'a> {
    pub fn new(growth_count: &'a State) -> Self {
        Self { growth_count }
    }
}

impl Drop for HandleSwitch<'_> {
    fn drop(&mut self) {
        self.growth_count.switched();
    }
}
