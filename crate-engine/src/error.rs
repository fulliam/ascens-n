use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum EcsError {
    #[error("component not registered: id={0}")]
    ComponentNotRegistered(u32),

    #[error("component already registered: '{0}'")]
    ComponentAlreadyRegistered(String),

    #[error("entity does not exist or is dead")]
    EntityNotFound,

    #[error("entity does not have component: id={0}")]
    ComponentNotFound(u32),

    #[error("archetype not found")]
    ArchetypeNotFound,

    #[error("invalid field count: expected {expected}, got {actual}")]
    FieldCountMismatch { expected: usize, actual: usize },

    #[error("invalid field type at index {index}")]
    FieldTypeMismatch { index: usize },

    #[error("system '{0}' not found")]
    SystemNotFound(String),

    #[error("resource not registered: id={0}")]
    ResourceNotRegistered(u32),

    #[error("event type not registered: id={0}")]
    EventNotRegistered(u32),
}

pub type EcsResult<T> = Result<T, EcsError>;
