mod wayland;

use clap::{Parser, Subcommand};
use wayland::WaylandClient;

#[derive(Parser)]
#[command(name = "wmctl")]
#[command(about = "Agnostic CLI for managing window managers")]
#[command(version)]
#[command(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clone, Subcommand)]
enum Command {
    #[command(about = "List the connected outputs")]
    ListOutputs {
        #[clap(short, long, help = "Show only the output name")]
        short: bool,
        #[clap(
            short,
            long,
            help = "Show the output in JSON",
            conflicts_with = "short"
        )]
        json: bool,
    },
    #[command(about = "Wait until an output gets connected or disconnected")]
    WatchForOutputChanges,
}

fn main() {
    let args = Args::parse();

    // We initialize the logger for the purpose of debugging.
    // Set `RUST_LOG=debug` to see extra debug information.
    env_logger::init();

    match args.cmd {
        Command::ListOutputs { short, json } => {
            let (wayland_client, _) = WaylandClient::new().unwrap();
            wayland_client.list_outputs(short, json);
        }
        Command::WatchForOutputChanges => {
            let (mut wayland_client, event_queue) = WaylandClient::new().unwrap();
            wayland_client.watch_for_output_changes(event_queue);
        }
    }
}
