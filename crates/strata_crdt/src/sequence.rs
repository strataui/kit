//! Replicated Growable Array (RGA) for collaborative rich-text and list editing.

use crate::clock::LamportClock;

/// Unique item in a collaborative sequence with causal origin pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgaNode<T> {
    pub id: LamportClock,
    pub after_id: Option<LamportClock>,
    pub value: T,
    pub deleted: bool,
}

/// Collaborative sequence CRDT supporting concurrent inserts and deletes with causal merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgaSequence<T> {
    nodes: Vec<RgaNode<T>>,
}

impl<T: Clone> Default for RgaSequence<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<T: Clone> RgaSequence<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts an item after `after_id` (or at the beginning if None).
    pub fn insert(&mut self, after_id: Option<LamportClock>, id: LamportClock, value: T) {
        if self.nodes.iter().any(|n| n.id == id) {
            return;
        }
        let new_node = RgaNode {
            id,
            after_id,
            value,
            deleted: false,
        };
        self.insert_node(new_node);
    }

    fn insert_node(&mut self, new_node: RgaNode<T>) {
        let id = new_node.id;
        let start_pos = match new_node.after_id {
            Some(target) => match self.nodes.iter().position(|n| n.id == target) {
                Some(pos) => pos + 1,
                None => 0,
            },
            None => 0,
        };

        // Skip sibling subtrees whose root has a higher Lamport timestamp than `id`
        let mut insert_pos = start_pos;
        while insert_pos < self.nodes.len() {
            let cur = &self.nodes[insert_pos];
            if cur.after_id == new_node.after_id {
                if cur.id > id {
                    insert_pos += 1;
                    // Also skip descendants attached under this higher-timestamp sibling
                    while insert_pos < self.nodes.len()
                        && self.is_descendant_of(self.nodes[insert_pos].after_id, cur.id)
                    {
                        insert_pos += 1;
                    }
                } else {
                    break;
                }
            } else if new_node
                .after_id
                .is_some_and(|p| self.is_descendant_of(cur.after_id, p))
            {
                insert_pos += 1;
            } else {
                break;
            }
        }
        self.nodes.insert(insert_pos, new_node);
    }

    fn is_descendant_of(&self, mut parent: Option<LamportClock>, ancestor: LamportClock) -> bool {
        let mut steps = 0;
        while let Some(pid) = parent {
            if pid == ancestor {
                return true;
            }
            steps += 1;
            if steps > self.nodes.len() {
                break;
            }
            parent = self
                .nodes
                .iter()
                .find(|n| n.id == pid)
                .and_then(|n| n.after_id);
        }
        false
    }

    /// Tombstones an element with the given ID.
    pub fn delete(&mut self, id: LamportClock) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.deleted = true;
            true
        } else {
            false
        }
    }

    /// Returns non-deleted elements in sequence order.
    pub fn read(&self) -> Vec<T> {
        self.nodes
            .iter()
            .filter(|n| !n.deleted)
            .map(|n| n.value.clone())
            .collect()
    }

    /// Merges another RGA sequence state while preserving causal parent-chain order.
    pub fn merge(&mut self, other: &RgaSequence<T>) {
        // First apply tombstones for existing nodes and collect missing nodes.
        let mut pending: Vec<&RgaNode<T>> = Vec::new();
        for other_node in &other.nodes {
            if let Some(local_node) = self.nodes.iter_mut().find(|n| n.id == other_node.id) {
                if other_node.deleted {
                    local_node.deleted = true;
                }
            } else {
                pending.push(other_node);
            }
        }

        // Insert nodes whose causal parent (`after_id`) is already present until convergence.
        while !pending.is_empty() {
            let mut progressed = false;
            let mut deferred = Vec::new();
            for node in pending {
                let parent_ready = match node.after_id {
                    None => true,
                    Some(pid) => self.nodes.iter().any(|n| n.id == pid),
                };
                if parent_ready {
                    self.insert_node(node.clone());
                    progressed = true;
                } else {
                    deferred.push(node);
                }
            }
            if !progressed {
                // Fallback for truncated history where parent tombstone was pruned
                for node in deferred {
                    self.insert_node(node.clone());
                }
                break;
            }
            pending = deferred;
        }
    }
}
