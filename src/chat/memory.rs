use std::collections::VecDeque;

use crate::chat_gpt_api::specification::Message;

pub(crate) trait Memory: Send + Clone {
    fn get(&self) -> Vec<Message>;
    fn add(&mut self, message: Message);
    fn clear(&mut self);
}

pub(crate) struct FiniteQueueMemory {
    pub(crate) memories: VecDeque<Message>,
    pub(crate) max_size: usize,
}

impl Memory for FiniteQueueMemory {
    fn get(&self) -> Vec<Message> {
        // Copy the memories as Vec
        self.memories.iter().cloned().collect()
    }

    fn add(&mut self, message: Message) {
        self.memories.push_back(message);
        while self.memories.len() > self.max_size {
            self.memories.pop_front();
        }
    }

    fn clear(&mut self) {
        self.memories.clear();
    }
}

impl Clone for FiniteQueueMemory {
    fn clone(&self) -> Self {
        Self {
            memories: self.memories.clone(),
            max_size: self.max_size,
        }
    }
}
