use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    rc::Rc,
};

use crate::{
    analysis::EventEffect,
    types::{Event, EventType, Graph, Link, Node},
};

/// Represents a Network consisting of [`Node`]s as a [`HashMap`]. [`Link`]s
/// for any given node is represented as a [`HashSet`].
#[derive(Debug, Clone)]
pub struct Network {
    nodes: HashMap<String, Rc<Node>>,
    pub graph: HashMap<Rc<Node>, HashSet<Rc<Node>>>,
}

impl Network {
    /// Add a [`Node`] to the Network.
    fn add_node(&mut self, node: &str) -> Rc<Node> {
        if let Some(existing) = self.nodes.get(node) {
            Rc::clone(existing)
        } else {
            let rc = Rc::new(Node(node.to_string()));
            self.nodes.insert(node.to_string(), Rc::clone(&rc));
            self.graph
                .entry(Rc::clone(&rc))
                .or_insert_with(HashSet::new);
            rc
        }
    }

    /// Connect a [`Link`] inside of the Network.
    fn connect(&mut self, a: &Rc<Node>, b: &Rc<Node>) {
        self.graph
            .entry(Rc::clone(a))
            .or_insert_with(HashSet::new)
            .insert(Rc::clone(b));

        self.graph
            .entry(Rc::clone(b))
            .or_insert_with(HashSet::new)
            .insert(Rc::clone(a));
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
    fn neighbors(&self, node: &Node) -> Option<&HashSet<Rc<Node>>> {
        self.graph.get(node)
    }

    /// Apply an [`Event`] to the Network.
    pub fn apply_event(&mut self, event: &Event) -> EventEffect {
        let node = &self.add_node(&event.node.0);
        let target = &self.add_node(&event.target.0);

        match event.event_type {
            EventType::LinkDown => {
                self.disconnect(node, target);
                EventEffect::TopologyChanged {
                    event: event.clone(),
                }
            }
            EventType::LinkUp => {
                self.connect(node, target);
                EventEffect::TopologyChanged {
                    event: event.clone(),
                }
            }
            EventType::NodeUnreachable => {
                let reachable = self.can_reach(node, target);
                EventEffect::Observation {
                    event: event.clone(),
                    reachable,
                }
            }
            EventType::Degraded => EventEffect::Ignored,
        }
    }

    /// Check if a [`Node`] can reach another [`Node`].
    pub fn can_reach(&self, node: &Rc<Node>, target: &Rc<Node>) -> bool {
        let mut visited = HashSet::new();
        self.traverse(node, target, &mut visited)
    }

    /// Recursively traverse the Network using depth-first search, returning
    /// `true` if the target [`Node`] is reachable from the given [`Node`].
    fn traverse(
        &self,
        node: &Rc<Node>,
        target: &Rc<Node>,
        visited: &mut HashSet<Rc<Node>>,
    ) -> bool {
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
    fn from(graph: Graph) -> Self {
        let mut network: Self = Self {
            nodes: HashMap::new(),
            graph: HashMap::new(),
        };

        for node in &graph.nodes {
            let _ = network.add_node(&node.0);
        }

        for Link(a, b) in &graph.links {
            let a = network.add_node(&a.0);
            let b = network.add_node(&b.0);
            network.connect(&a, &b);
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
