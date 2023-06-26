use crate::{
    chat::memory::FiniteQueueMemory,
    chat_gpt_api::specification::{Function, Model},
    vector_db::vector_memories::VectorMemories,
};

pub(crate) struct ApiState<'a> {
    pub(crate) model: Model,
    pub(crate) context_memory: FiniteQueueMemory,
    pub(crate) vector_memories: VectorMemories<'a>,
    pub(crate) functions: Vec<Function>,
}
