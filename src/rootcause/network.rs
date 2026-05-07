use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
};

use serde::Deserialize;

use crate::rootcause::{
    analysis::{Candidate, ScoredEvidence},
    types::{Event, EventType, Graph, Link, Node},
};

#[derive(Debug)]
pub struct Network {
    pub graph: HashMap<Node, HashSet<Node>>,
}

impl Network {
    /// Create a Network from a given Graph
    pub fn new(graph: Graph) -> Self {
        let mut network: Self = Self {
            graph: HashMap::new(),
        };

        for node in &graph.nodes {
            network.add_node(node)
        }

        for Link(a, b) in &graph.links {
            network.connect(a, b);
        }

        network
    }

    /// Add a node to the Network
    fn add_node(&mut self, node: &Node) {
        self.graph.entry(node.clone()).or_insert_with(HashSet::new);
    }

    /// Connect two Nodes inside the Network
    fn connect(&mut self, a: &Node, b: &Node) {
        self.graph
            .entry(a.clone())
            .or_insert_with(HashSet::new)
            .insert(b.clone());

        self.graph
            .entry(b.clone())
            .or_insert_with(HashSet::new)
            .insert(a.clone());
    }

    /// Disconnect two Nodes from the Network
    fn disconnect(&mut self, a: &Node, b: &Node) {
        if let Some(neighbors) = self.graph.get_mut(a) {
            neighbors.remove(b);
        }
        if let Some(neighbors) = self.graph.get_mut(b) {
            neighbors.remove(a);
        }
    }

    /// Check if two Nodes are neighbors
    fn neighbors(&self, node: &Node) -> Option<&HashSet<Node>> {
        self.graph.get(node)
    }

    /// Apply an Event to the Network
    pub fn apply_event(&mut self, event: &Event) -> EventEffect {
        match event.event_type {
            EventType::LinkDown => {
                self.disconnect(&event.node, &event.target);
                EventEffect::TopologyChanged {
                    event: event.clone(),
                }
            }
            EventType::LinkUp => {
                self.connect(&event.node, &event.target);
                EventEffect::TopologyChanged {
                    event: event.clone(),
                }
            }
            EventType::NodeUnreachable => {
                let reachable = self.can_reach(&event.node, &event.target);
                EventEffect::Observation {
                    event: event.clone(),
                    reachable,
                }
            }
            EventType::Degraded => EventEffect::Ignored,
        }
    }

    /// Check if a Node can reach another Node
    pub fn can_reach(&self, node: &Node, target: &Node) -> bool {
        let mut visited = HashSet::new();
        self.traverse(node, target, &mut visited)
    }

    /// Recursively traverse the Network using DFS until target Node is visited, or not
    fn traverse(&self, node: &Node, target: &Node, visited: &mut HashSet<Node>) -> bool {
        if node == target {
            return true;
        }

        if !visited.insert(node.clone()) {
            return false;
        }

        if let Some(neighbors) = self.neighbors(node) {
            for neighbor in neighbors {
                if self.traverse(neighbor, target, visited) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum EventEffect {
    TopologyChanged { event: Event },
    Observation { event: Event, reachable: bool },
    Ignored,
}

impl EventEffect {
    pub fn candidate(&self) -> Option<ScoredEvidence> {
        match self {
            Self::Ignored => None,
            Self::TopologyChanged { event } => self.on_topology(event),
            Self::Observation { event, reachable } => self.on_observation(event, reachable),
        }
    }

    fn on_topology(&self, event: &Event) -> Option<ScoredEvidence> {
        if matches!(event.event_type, EventType::LinkDown) {
            let l = Link::new(event.node.clone(), event.target.clone());
            Some(ScoredEvidence::new(Candidate::Link(l), self.clone(), 1))
        } else {
            None
        }
    }

    fn on_observation(&self, event: &Event, reachable: &bool) -> Option<ScoredEvidence> {
        let t = event.target.clone();
        let delta = if *reachable { 2 } else { 1 };
        Some(ScoredEvidence::new(Candidate::Node(t), self.clone(), delta))
    }
}

impl Display for EventEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventEffect::TopologyChanged { event } => write!(f, "{event}"),
            EventEffect::Observation { event, reachable } => {
                write!(f, "{event}\n  Topology disagreed: {reachable}")
            }
            EventEffect::Ignored => write!(f, ""),
        }
    }
}
