use clap::Parser;
use std::path::PathBuf;

use crate::{
    analysis::RootCauseTracker,
    network::Network,
    types::{Event, Graph},
};

mod analysis;
mod network;
mod types;

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
    let mut network = Network::new(graph);
    let initial_network = network.clone();

    let mut events = Event::parse_file(cli.event)?;
    events.sort_by_key(|e| e.timestamp);

    let mut tracker = RootCauseTracker::new();
    for event in events {
        let effect = network.apply_event(&event);
        let candidate = effect.candidate();
        if let Some(candidate) = candidate
            && !candidate.is_empty()
        {
            candidate.iter().for_each(|c| tracker.record(c));
        }
    }

    print_network_comparison(&initial_network, &network);

    let causes = tracker.get_causes()?;
    Ok(println!("{causes}"))
}

fn print_network_comparison(left: &Network, right: &Network) {
    if left.graph.is_empty() || right.graph.is_empty() {
        return;
    }

    let left = format!("{left}")
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let right = format!("{right}")
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let width = left.iter().map(|l| l.len()).max().unwrap_or(0);

    println!(
        "{:<width$} | {}",
        "Initial Network State",
        "Modified Network State",
        width = width
    );

    for i in 0..left.len().max(right.len()) {
        let l = left.get(i).map(String::as_str).unwrap_or("");
        let r = right.get(i).map(String::as_str).unwrap_or("");

        println!("{:<width$} | {}", l, r, width = width);
    }
    println!("")
}
