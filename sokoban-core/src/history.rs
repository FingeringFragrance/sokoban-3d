use crate::grid::GridSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveHistory {
    records: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub snapshot: GridSnapshot,
    pub direction: crate::types::Direction,
}

impl MoveHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, snapshot: GridSnapshot, direction: crate::types::Direction) {
        self.redo_stack.clear();
        self.records.push(HistoryEntry {
            snapshot,
            direction,
        });
    }

    pub fn pop(&mut self) -> Option<GridSnapshot> {
        let entry = self.records.pop()?;
        let snapshot = entry.snapshot.clone();
        self.redo_stack.push(entry);
        Some(snapshot)
    }

    pub fn redo(&mut self) -> Option<GridSnapshot> {
        let entry = self.redo_stack.pop()?;
        let snapshot = entry.snapshot.clone();
        self.records.push(entry);
        Some(snapshot)
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn peek_direction(&self) -> Option<crate::types::Direction> {
        self.records.last().map(|e| e.direction)
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.redo_stack.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}
