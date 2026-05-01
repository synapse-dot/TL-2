use std::collections::HashMap;

use crate::ast::Block;

#[derive(Debug, Clone)]
pub struct FunctionVersion {
    pub start_ms: i64,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionStore {
    versions: HashMap<String, Vec<FunctionVersion>>,
}

impl FunctionStore {
    pub fn define(&mut self, name: String, start_ms: i64, params: Vec<String>, body: Block) {
        let entry = self.versions.entry(name).or_default();
        entry.push(FunctionVersion {
            start_ms,
            params,
            body,
        });
        entry.sort_by_key(|v| v.start_ms);
    }

    pub fn active_at(&self, name: &str, t_ms: i64) -> Option<&FunctionVersion> {
        self.versions
            .get(name)?
            .iter()
            .rev()
            .find(|v| v.start_ms <= t_ms)
    }

    pub fn all(&self) -> &HashMap<String, Vec<FunctionVersion>> {
        &self.versions
    }
}
