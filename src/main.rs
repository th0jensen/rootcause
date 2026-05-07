use clap::Parser;
use std::path::PathBuf;

use crate::rootcause::{
    analysis::{RootCause, RootCauseTracker},
    network::Network,
    types::{Event, Graph},
};

mod rootcause;

#[derive(Parser)]
#[command(
    name = "rootcause",
    version,
    about = "Identify the root cause in a dynamic IP network"
)]
#[command(arg_required_else_help = true)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    event: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let graph = Graph::parse_file(cli.input)?;
    let mut events = Event::parse_file(cli.event)?;
    let mut network = Network::new(graph);
    events.sort_by_key(|event| event.timestamp);

    let mut tracker = RootCauseTracker::new();
    for event in &events {
        let effect = network.apply_event(event);
        let candidate = effect.candidate();
        if let Some(candidate) = candidate {
            tracker.record(candidate);
        }
    }

    if let Some(cause) = RootCause::get_cause(tracker) {
        println!("{cause}");
    } else {
        println!("No cause found!")
    }

    Ok(())
}
