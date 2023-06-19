use std::collections::VecDeque;

use crate::chat_gpt::specification::Message;

pub(crate) trait Memory {
    fn get_memories(&self) -> Vec<Message>;
    fn add_memory(&mut self, message: Message);
    fn clear_memories(&mut self);
}

pub(crate) struct FiniteQueueMemory {
    memories: VecDeque<Message>,
    max_size: usize,
}

impl Memory for FiniteQueueMemory {
    fn get_memories(&self) -> Vec<Message> {
        // Copy the memories as Vec
        self.memories.iter().cloned().collect()
    }

    fn add_memory(&mut self, message: Message) {
        self.memories.push_back(message);
        while self.memories.len() > self.max_size {
            self.memories.pop_front();
        }
    }

    fn clear_memories(&mut self) {
        self.memories.clear();
    }
}
