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
    let initial_network = network.clone();
    events.sort_by_key(|e| e.timestamp);

    let mut tracker = RootCauseTracker::new();
    for event in events {
        let effect = network.apply_event(&event);
        let candidate = effect.candidate();
        if let Some(candidate) = candidate {
            candidate.iter().for_each(|c| tracker.record(c));
        }
    }

    print_network_comparison(&initial_network, &network);

    let causes = RootCause::get_causes(tracker);

    let top_score = causes
        .first()
        .map(|c| c.score)
        .ok_or_else(|| anyhow::anyhow!("no causes found for network"))?;

    let mut most_likely = causes
        .iter()
        .filter(|c| c.score == top_score)
        .collect::<Vec<_>>();

    print_results("Most", &mut most_likely);

    let mut less_likely = causes
        .iter()
        .filter(|c| c.score != top_score && c.score > 0.0)
        .collect::<Vec<_>>();

    print_results("Less", &mut less_likely);

    let mut symptoms = causes.iter().filter(|c| c.score == 0.0).collect::<Vec<_>>();
    print_results("Symptoms", &mut symptoms);

    Ok(())
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

fn print_results(label: &str, causes: &mut [&RootCause]) {
    if causes.is_empty() {
        return;
    }

    if label == "Symptoms" {
        println!("Observed downstream symptoms:")
    } else {
        let plural = if causes.len() > 1 { "s" } else { "" };
        println!("{label} likely root cause{plural}:\n");
    }

    for cause in causes {
        println!("{cause}");
    }
}
