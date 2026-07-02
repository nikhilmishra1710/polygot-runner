use crate::runtime::RuntimeCommand;

#[derive(Debug)]
pub struct ExecutionPlan {
    pub compile: Option<RuntimeCommand>,
    pub execute: RuntimeCommand,
}
