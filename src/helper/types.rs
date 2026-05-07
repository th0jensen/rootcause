use jiff::Timestamp;
use serde::{Deserialize, Deserializer, de};

#[derive(Debug, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[derive(Debug, Deserialize)]
pub struct Node(String);

#[derive(Debug, Deserialize)]
pub struct Link(Node, Node);

#[derive(Debug, Deserialize)]
pub struct Event {
    pub node: Node,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub target: Node,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: Timestamp,
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let ms: i64 = Deserialize::deserialize(deserializer)?;
    Timestamp::from_millisecond(ms).map_err(de::Error::custom)
}

#[derive(Debug, Deserialize)]
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
