use core::time;
use std::collections::HashSet;

use lib::{search::perform_search, models::Person};
use tracing_subscriber::{Registry, EnvFilter, util::SubscriberInitExt, layer::SubscriberExt};
use tracing_tree::HierarchicalLayer;
use tracing_unwrap::ResultExt;
use clap::{Parser, Subcommand, Args};

mod lib;

#[derive(Args,Debug)]
struct Common {
    lastname: String,

    #[clap(short,long)]
    firstname: Option<String>,

    #[clap(short,long,default_value_t = 50)]
    limit: usize,
}
#[derive(Parser,Debug)]
#[clap(author,version,about,long_about=None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand,Debug)]
enum Commands {
    Search { 
        #[clap(flatten)]
        common: Common
    },
    Watch { 
        #[clap(flatten)]
        common: Common,
        interval: usize }
}

fn main() {
    setup_tracing();

    color_eyre::install().expect("Installing coloreyre failed");

    let args = Cli::parse();

    match &args.command {
        Commands::Watch { common, interval } => watch(common, interval),
        Commands::Search { common } => search(common)
    };
}

fn watch(common: &Common, interval: &usize) {
    let current = search_with_args(common);
    let curr_hash: HashSet<_> = current.iter().collect();

    std::thread::sleep(time::Duration::from_secs(*interval as u64));
    let new = search_with_args(common);
    let new_hash: HashSet<_> = new.iter().collect();

    println!("{:?}", &curr_hash.symmetric_difference(&new_hash).collect::<Vec<_>>())
}

fn search_with_args(common: &Common) -> Vec<Person> {
    perform_search(&common.lastname, common.limit, common.firstname.clone())
}

fn search(common: &Common) {
    let persons = search_with_args(common);

    let serialized = serde_json::to_string_pretty(&persons).unwrap_or_log();
    println!("{}", serialized);
}

fn setup_tracing() {
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();
}
