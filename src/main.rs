use lib::search::perform_search;
use tracing_subscriber::{Registry, EnvFilter, util::SubscriberInitExt, layer::SubscriberExt};
use tracing_tree::HierarchicalLayer;
use tracing_unwrap::ResultExt;
use clap::Parser;

mod lib;

#[derive(Parser,Debug)]
#[clap(author,version,about,long_about=None)]
struct Args {
    lastname: String,

    #[clap(short,long)]
    firstname: Option<String>,

    #[clap(short,long,default_value_t = 50)]
    limit: usize,
}

fn main() {
    setup_tracing();

    color_eyre::install().expect("Installing coloreyre failed");

    let args = Args::parse();

    let persons = perform_search(&args.lastname, args.limit, args.firstname);

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
