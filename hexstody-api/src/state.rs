use std::collections::HashMap;

pub struct RuntimeState {
    pub challenges: HashMap<String, String>
}

impl RuntimeState {
    pub fn new() -> Self{
        RuntimeState{challenges: HashMap::new()}
    }
}