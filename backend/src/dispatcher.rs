use std::sync::{Arc, Mutex};
use crate::retriever::Retriever;

#[derive(Debug)]
pub struct Dispatcher {
    memory: Arc<Mutex<Vec<String>>>, // Replace with your actual memory type
    retriever: Retriever,
}

impl Dispatcher {
    pub fn new(memory: Arc<Mutex<Vec<String>>>, retriever: Retriever) -> Self {
        Dispatcher { memory, retriever }
    }

    pub fn handle_event(&self, event: notify::Event) {
        println!("Handling event: {:?}", event);
        // Use self.retriever to interpret the event
        // Update memory or trigger actions
    }
}
