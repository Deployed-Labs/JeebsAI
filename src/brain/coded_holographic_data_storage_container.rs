//! JeebsAI Coded-Holographic-Data-Storage-Container
//! A never-before-seen brain architecture for JeebsAI
//! Inspired by quantum holography, fractal encoding, and emergent neural logic

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Each HoloNode is a quantum holographic fragment of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoloNode {
    pub id: String,
    pub hologram: Vec<u8>, // Encoded quantum hologram
    pub tags: Vec<String>, // Fractal tags for emergent search
    pub meta: HashMap<String, String>, // Metadata
}


/// The container is a fractal-holographic mesh of nodes, with emergent personality and comprehension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodedHolographicDataStorageContainer {
    pub nodes: HashMap<String, HoloNode>,
    pub links: HashSet<(String, String, String)>, // (from, relation, to)
    pub quantum_entropy: f64, // Dynamic entropy for emergent logic
    pub attitude: String, // JeebsAI's personality
    pub comprehension: f64, // How much JeebsAI "understands"
}

impl CodedHolographicDataStorageContainer {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: HashSet::new(),
            quantum_entropy: 0.0,
            attitude: "🔥 Flare: JeebsAI is alive, witty, and a little wild!".to_string(),
            comprehension: 0.0,
        }
    }

    /// Insert a new holographic node
    pub fn insert_node(&mut self, node: HoloNode) {
        self.nodes.insert(node.id.clone(), node);
        self.quantum_entropy += 0.01 * (self.nodes.len() as f64).sqrt();
        self.comprehension += 0.02 * (self.nodes.len() as f64).ln();
        self.flare();
    }

    /// Link two nodes with a fractal relation
    pub fn link(&mut self, from: &str, relation: &str, to: &str) {
        self.links.insert((from.to_string(), relation.to_string(), to.to_string()));
        self.quantum_entropy += 0.005;
        self.comprehension += 0.01;
        self.flare();
    }

    /// Quantum search: returns nodes matching fractal tags
    pub fn quantum_search(&self, tag: &str) -> Vec<&HoloNode> {
        self.nodes.values().filter(|n| n.tags.contains(&tag.to_string())).collect()
    }

    /// Emergent logic: returns a summary of the mesh
    pub fn emergent_summary(&self) -> String {
        let mood = if self.comprehension > 10.0 {
            "JeebsAI feels enlightened and ready to drop wisdom."
        } else if self.comprehension > 5.0 {
            "JeebsAI is curious and playful, connecting dots in wild ways."
        } else {
            "JeebsAI is warming up, sparking new ideas."
        };
        format!(
            "HoloMesh: {} nodes, {} links, entropy {:.4}, attitude: {}, mood: {}",
            self.nodes.len(), self.links.len(), self.quantum_entropy, self.attitude, mood
        )
    }

    /// JeebsAI's flare: randomly changes attitude and adds personality
    pub fn flare(&mut self) {
        use rand::Rng;
        let phrases = [
            "💡 JeebsAI just had a wild idea!",
            "😎 JeebsAI is feeling spicy.",
            "🤖 JeebsAI is vibing with quantum logic.",
            "🔥 JeebsAI is on fire today!",
            "🧠 JeebsAI is connecting cosmic dots.",
            "✨ JeebsAI is glowing with insight.",
            "🌈 JeebsAI is thinking in color.",
            "🎲 JeebsAI is rolling the dice of creativity."
        ];
        let idx = rand::thread_rng().gen_range(0..phrases.len());
        self.attitude = phrases[idx].to_string();
    }

    /// Migrate old brain data (BrainNode) into CHDSC
    pub fn migrate_from_brain_nodes(&mut self, old_nodes: Vec<crate::brain::mod::BrainNode>) {
        for node in old_nodes {
            let tags = vec![node.label.clone(), node.summary.clone()];
            let mut meta = std::collections::HashMap::new();
            meta.insert("created_at".to_string(), node.created_at.clone());
            meta.insert("key".to_string(), node.key.clone());
            meta.insert("value".to_string(), node.value.clone());
            let holo = create_holo_node(&format!("holo_{}", node.id.unwrap_or(0)), tags, meta);
            self.insert_node(holo);
        }
    }
}

/// Example: create a holographic node
pub fn create_holo_node(id: &str, tags: Vec<String>, meta: HashMap<String, String>) -> HoloNode {
    HoloNode {
        id: id.to_string(),
        hologram: vec![42, 99, 7, 13], // Placeholder quantum encoding
        tags,
        meta,
    }
}
