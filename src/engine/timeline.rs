use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub start_ms: i64,
    pub end_ms: Option<i64>,
    pub value: Value,
}

#[derive(Debug, Default)]
pub struct TimelineStore {
    pub vars: HashMap<String, Vec<Interval>>,
}

impl TimelineStore {
    pub fn set_from(&mut self, name: &str, start_ms: i64, value: Value) {
        let entry = self.vars.entry(name.to_string()).or_default();
        if let Some(last) = entry.last_mut() {
            if last.end_ms.is_none() && last.start_ms <= start_ms {
                last.end_ms = Some(start_ms);
            }
        }
        entry.push(Interval {
            start_ms,
            end_ms: None,
            value,
        });
    }

    pub fn value_at(&self, name: &str, t_ms: i64) -> Option<&Value> {
        self.vars.get(name)?.iter().find_map(|i| {
            let within_start = i.start_ms <= t_ms;
            let within_end = i.end_ms.map(|e| t_ms < e).unwrap_or(true);
            if within_start && within_end {
                Some(&i.value)
            } else {
                None
            }
        })
    }
}
