use crate::ComponentValue;

/// A component value bundle for spawning entities
#[derive(Debug, Clone)]
pub struct ComponentInstance {
    pub component_id: u32,
    pub values: Vec<ComponentValue>,
}

impl ComponentInstance {
    pub fn new(component_id: u32, values: Vec<ComponentValue>) -> Self {
        Self { component_id, values }
    }
}
