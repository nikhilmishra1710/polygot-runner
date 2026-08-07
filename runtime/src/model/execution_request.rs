use crate::model::ResourceLimits;

use super::{Language, SourceFile};

#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub language: Language,
    pub files: Vec<SourceFile>,
    pub stdin: Vec<u8>,
    pub limits: ResourceLimits,
}
