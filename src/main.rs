use clap::Parser;
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::helper::{
    parse::{parse_events, parse_input},
    types::Graph,
};

mod helper;

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

    let graph = parse_input(cli.input)?;
    let events = parse_events(cli.event)?;
    println!("{:?}\n{:?}", graph, events);
    Ok(())
}
