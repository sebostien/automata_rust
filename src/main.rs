use std::process::ExitCode;

use clap::{Parser, Subcommand};

use automata_rust::{self, graph_display::DiGraph, language::Language};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Svg {
        #[arg(long)]
        nfa: bool,
        #[arg(long)]
        dfa: bool,
        input: String,
    },
    Table {
        #[arg(long)]
        nfa: bool,
        input: String,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("{e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut svg = None;
    let mut table = None;

    match args.command {
        Commands::Svg { nfa, dfa, input } => {
            if nfa {
                let nfa = automata_rust::nfa::NFA::try_from_language(input)?;
                let graph: DiGraph = (&nfa).into();
                svg = Some(graph.to_string());
            } else if dfa {
                let nfa = automata_rust::nfa::NFA::try_from_language(input)?;
                let dfa = automata_rust::dfa::DFA::from(nfa);
                let graph: DiGraph = (&dfa).into();
                svg = Some(graph.to_string());
            }
        }
        Commands::Table { nfa, input } => {
            if nfa {
                table = Some(automata_rust::nfa::NFA::try_from_language(input)?.to_string());
            } else {
                return Err("Exactly one graph representation must be chosen!".into());
            }
        }
    }

    if let Some(svg) = svg {
        std::fs::write("./graph.svg", svg).expect("Could not write data to file!");
    }

    if let Some(table) = table {
        println!("{table}");
    }

    Ok(())
}
