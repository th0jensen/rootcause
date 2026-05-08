use std::{
    fmt::{self, Display},
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use jiff::Timestamp;
use serde::{Deserialize, Deserializer, de};

/// Represents a [`crate::Network`] as a graph consisting of Nodes and Links.
#[derive(Debug, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

impl Graph {
    /// Parse a [`Graph`] from an input JSON file
    pub fn parse_file(file: PathBuf) -> anyhow::Result<Self> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        let graph: Graph = serde_json::from_reader(reader)?;
        Ok(graph)
    }
}

/// Represents an Event that has occurred in the [`crate::Network`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Event {
    pub node: Node,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub target: Node,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: Timestamp,
}

impl Event {
    /// Parse an [`Event`] from an input JSON file.
    pub fn parse_file(file: PathBuf) -> anyhow::Result<Vec<Self>> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        let events: Vec<Event> = serde_json::from_reader(reader)?;
        Ok(events)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Event {
            node,
            event_type,
            target,
            timestamp,
        } = self;

        if let Ok(date) = timestamp.in_tz("Europe/Oslo") {
            write!(
                f,
                "Reported by {node} at {}:\n  Target: {target}, Type: {event_type}",
                date.strftime("%d/%m/%Y %H:%M:%S")
            )
        } else {
            write!(
                f,
                "Reported by {node} at {timestamp}:\n  Target: {target}, Type: {event_type}"
            )
        }
    }
}

/// Convert the timestamp field from EPOCH_MILLISECONDS to [`Timestamp`] upon
/// deserialization. Will throw an error if the timestamp is not a valid
/// POSIX epoch type. For example by being out of range.
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let ms: i64 = Deserialize::deserialize(deserializer)?;
    Timestamp::from_millisecond(ms).map_err(de::Error::custom)
}

/// Represents any given Node in the [`crate::Network`].
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Node(pub String);

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

/// Represents a bidirectional connection between two Nodes.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Link(pub Node, pub Node);

impl Link {
    /// Sorts entries in Link alphabetically as to not create duplicate
    /// candidates for the [`crate::Network`].
    pub fn new(a: Node, b: Node) -> Self {
        if a.0 <= b.0 { Self(a, b) } else { Self(b, a) }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Link({}, {})", self.0, self.1)
    }
}

/// Represents the type of an [`Event`] that may occur.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum EventType {
    #[serde(rename = "LINK_UP")]
    LinkUp,
    #[serde(rename = "LINK_DOWN")]
    LinkDown,
    #[serde(rename = "NODE_UNREACHABLE")]
    NodeUnreachable,
    #[serde(rename = "DEGRADED")]
    Degraded,
}

impl Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::LinkUp => write!(f, "LINK_UP"),
            EventType::LinkDown => write!(f, "LINK_DOWN"),
            EventType::NodeUnreachable => write!(f, "NODE_UNREACHABLE"),
            EventType::Degraded => write!(f, "DEGRADED"),
        }
    }
}
