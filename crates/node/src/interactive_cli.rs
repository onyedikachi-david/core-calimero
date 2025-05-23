#![allow(clippy::print_stdout, reason = "Acceptable for CLI")]
#![allow(
    clippy::multiple_inherent_impl,
    reason = "TODO: Check if this is necessary"
)]

mod applications;
pub mod call;
pub mod common;
pub mod context;
pub mod peers;
pub mod state;
pub mod store;
pub mod webui;

use clap::{Parser, Subcommand};

use crate::Node;

#[derive(Debug, Parser)]
#[command(multicall = true, bin_name = "{repl}")]
#[non_exhaustive]
pub struct RootCommand {
    #[command(subcommand)]
    pub action: SubCommand,
}

#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum SubCommand {
    #[command(alias = "app")]
    Application(applications::ApplicationCommand),
    Call(call::CallCommand),
    Context(context::ContextCommand),
    Peers(peers::PeersCommand),
    // Store(store::StoreCommand),
    State(state::StateCommand),
    #[command(name = "webui")]
    WebUI(webui::WebUICommand),
}

pub async fn handle_line(node: &mut Node, line: String) -> eyre::Result<()> {
    let mut args = line.split_whitespace().peekable();

    if args.peek().is_none() {
        return Ok(());
    }

    let command = match RootCommand::try_parse_from(args) {
        Ok(command) => command,
        Err(err) => {
            println!("{err}");
            return Ok(());
        }
    };

    match command.action {
        SubCommand::Application(application) => application.run(node).await?,
        SubCommand::Call(call) => call.run(node).await?,
        SubCommand::Context(context) => context.run(node).await?,
        SubCommand::Peers(peers) => peers.run(node).await?,
        SubCommand::State(state) => state.run(node)?,
        SubCommand::WebUI(cmd) => cmd.run(node)?,
        // SubCommand::Store(store) => store.run(node)?,
    }

    Ok(())
}
