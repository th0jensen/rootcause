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
            tracker.record(candidate);
        }
    }

    print_network_comparison(&initial_network, &network);

    if let Some(causes) = RootCause::get_causes(tracker) {
        let top_score = causes[0].score;

        let most_likely = causes
            .iter()
            .filter(|c| c.score == top_score)
            .collect::<Vec<_>>();

        print_causes("Most", &most_likely);

        let less_likely = causes
            .iter()
            .filter(|c| c.score != top_score)
            .collect::<Vec<_>>();

        print_causes("Less", &less_likely);
    } else {
        println!("no plausible causes found from events")
    }

    Ok(())
}

fn print_network_comparison(left: &Network, right: &Network) {
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

fn print_causes(label: &str, causes: &[&RootCause]) {
    if causes.is_empty() {
        return;
    }

    let plural = if causes.len() > 1 { "s" } else { "" };
    println!("{label} likely root cause{plural}:\n");

    for cause in causes {
        println!("{cause}");
    }
}
