use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
};

use crate::{
    analysis::EventEffect,
    types::{Event, EventType, Graph, Link, Node},
};

/// Represents a Network consisting of [`Node`]s as a [`HashMap`]. [`Link`]s
/// for any given node is represented as a [`HashSet`].
#[derive(Debug, Clone)]
pub struct Network {
    pub graph: HashMap<Node, HashSet<Node>>,
}

impl Network {
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

impl From<Graph> for Network {
    /// Create a Network from a given [`Graph`].
    fn from(graph: Graph) -> Self {
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
