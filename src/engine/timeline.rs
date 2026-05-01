use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictPolicy {
    LastWriteWins,
    Error,
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
    pub fn set_from(
        &mut self,
        name: &str,
        start_ms: i64,
        value: Value,
        policy: ConflictPolicy,
    ) -> Result<(), String> {
        let entry = self.vars.entry(name.to_string()).or_default();

        if let Some(last) = entry.last() {
            let overlaps =
                last.end_ms.map(|e| start_ms < e).unwrap_or(true) && start_ms >= last.start_ms;
            if overlaps && start_ms == last.start_ms && last.value != value {
                if policy == ConflictPolicy::Error {
                    return Err(format!(
                        "conflict on '{name}' at {start_ms}ms: existing={:?}, new={:?}",
                        last.value, value
                    ));
                }
            }
        }

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

        Ok(())
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
