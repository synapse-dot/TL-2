use std::collections::VecDeque;
use std::collections::HashMap;

pub struct Process {
    pub pid: u32,
    pub mailbox: VecDeque<String>,
    pub current_fn: Option<String>,
    pub local_scope: HashMap<String, String>,
}

pub struct ProcessStore {
    pub processes: HashMap<u32, Process>,
}