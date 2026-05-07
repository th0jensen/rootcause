use std::{fs::File, io::BufReader, path::PathBuf};

use crate::helper::types::{Event, Graph};

pub fn parse_input(file: PathBuf) -> anyhow::Result<Graph> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let graph: Graph = serde_json::from_reader(reader)?;
    Ok(graph)
}

pub fn parse_events(file: PathBuf) -> anyhow::Result<Vec<Event>> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let events: Vec<Event> = serde_json::from_reader(reader)?;
    Ok(events)
}
