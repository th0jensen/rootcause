use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
};

use serde::Deserialize;

use crate::rootcause::{
    analysis::{Candidate, ScoredEvidence},
    types::{Event, EventType, Graph, Link, Node},
};

/// Represents a Network consisting of [`Node`]s as a [`HashMap`]. [`Link`]s
/// for any given node is represented as a [`HashSet`].
#[derive(Debug, Clone)]
pub struct Network {
    pub graph: HashMap<Node, HashSet<Node>>,
}

impl Network {
    /// Create a Network from a given [`Graph`].
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

    /// Add a [`Node`] to the Network.
    fn add_node(&mut self, node: &Node) {
        self.graph.entry(node.clone()).or_insert_with(HashSet::new);
    }

    /// Connect a [`Link`] inside of the Network.
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

    /// Disconnect a [`Link`] from the Network.
    fn disconnect(&mut self, a: &Node, b: &Node) {
        if let Some(neighbors) = self.graph.get_mut(a) {
            neighbors.remove(b);
        }
        if let Some(neighbors) = self.graph.get_mut(b) {
            neighbors.remove(a);
        }
    }

    /// Check if two Nodes are neighbors.
    fn neighbors(&self, node: &Node) -> Option<&HashSet<Node>> {
        self.graph.get(node)
    }

    /// Apply an [`Event`] to the Network.
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

    /// Check if a [`Node`] can reach another [`Node`].
    pub fn can_reach(&self, node: &Node, target: &Node) -> bool {
        let mut visited = HashSet::new();
        self.traverse(node, target, &mut visited)
    }

    /// Recursively traverse the Network using depth-first search, returning
    /// `true` if the target [`Node`] is reachable from the given [`Node`].
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

impl Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut nodes = self.graph.iter().collect::<Vec<_>>();
        nodes.sort_by(|a, b| a.0.cmp(b.0));

        for node in nodes {
            let (node, neighbors) = node;
            let mut neighbors = neighbors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();

            neighbors.sort_by(|a, b| a.cmp(b));
            writeln!(f, "{node} -> [{}]", neighbors.join(", "))?;
        }

        Ok(())
    }
}

/// Represents the effect of an [`Event`] on a [`Network`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum EventEffect {
    TopologyChanged { event: Event },
    Observation { event: Event, reachable: bool },
    Ignored,
}

impl EventEffect {
    /// Evaluate this [`EventEffect`] as a potential root cause candidate for
    /// [`crate::RootCauseTracker`].
    pub fn candidate(&self) -> Option<Vec<ScoredEvidence>> {
        match self {
            Self::Ignored => None,
            Self::TopologyChanged { event } => self.on_topology(event),
            Self::Observation { event, reachable } => self.on_observation(event, reachable),
        }
    }

    /// Score an [`EventType::LinkDown`] topology change as evidence against
    /// the affected [`Link`] and target [`Node`].
    fn on_topology(&self, event: &Event) -> Option<Vec<ScoredEvidence>> {
        if matches!(event.event_type, EventType::LinkDown) {
            let l = Link::new(event.node.clone(), event.target.clone());
            let t = event.target.clone();
            Some(vec![
                ScoredEvidence::new(Candidate::Link(l), self.clone(), 1),
                ScoredEvidence::new(Candidate::Node(t), self.clone(), 1),
            ])
        } else {
            None
        }
    }

    /// Score an [`EventType::NodeUnreachable`] observation as evidence against
    /// the target [`Node`]. A higher delta is assigned when the topology
    /// confirms the node is still reachable, suggesting the report is
    /// unexpected and more significant.
    fn on_observation(&self, event: &Event, reachable: &bool) -> Option<Vec<ScoredEvidence>> {
        let t = event.target.clone();
        let delta = if *reachable { 2 } else { 1 };
        Some(vec![ScoredEvidence::new(
            Candidate::Node(t),
            self.clone(),
            delta,
        )])
    }
}

impl Display for EventEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventEffect::TopologyChanged { event } => write!(f, "{event}"),
            EventEffect::Observation { event, reachable } => {
                write!(f, "{event}\n  Topology still reachable: {reachable}")
            }
            EventEffect::Ignored => write!(f, ""),
        }
    }
}
