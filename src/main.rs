use anyhow::Result;
use clap::Parser;

use rdf4j_cli::cli::{Cli, Commands};
use rdf4j_cli::client::Rdf4jClient;
use rdf4j_cli::commands;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Rdf4jClient::new(&cli.server)?;

    match &cli.command {
        Commands::Server(args) => commands::server::handle(&client, &args.command, cli.format),
        Commands::Repo(args) => commands::repo::handle(&client, &args.command, cli.format),
        Commands::Query(args) => commands::query::handle(&client, args, cli.format),
        Commands::Update(args) => commands::update::handle(&client, args),
        Commands::Namespace(args) => {
            commands::namespace::handle(&client, &args.repo_id, &args.command, cli.format)
        }
        Commands::Statement(args) => {
            commands::statement::handle(&client, &args.repo_id, &args.command, cli.format)
        }
        Commands::Upload(args) => commands::upload::handle(&client, args),
    }
}
