use crate::{
    chat_gpt_api::memory::FiniteQueueMemory,
    chat_gpt_api::specification::{Function, Model},
};

pub(crate) struct ApiState {
    pub(crate) model: Model,
    pub(crate) prompt: String,
    pub(crate) context_memory: FiniteQueueMemory,
    pub(crate) functions: Vec<Function>,
}
