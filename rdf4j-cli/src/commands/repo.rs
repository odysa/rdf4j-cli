use anyhow::Result;

use crate::cli::{OutputFormat, RepoCommand};
use crate::output;
use rdf4j_rs::Rdf4jClient;

pub fn handle(client: &Rdf4jClient, cmd: &RepoCommand, format: OutputFormat) -> Result<()> {
    match cmd {
        RepoCommand::List => {
            let json = client.list_repos()?;
            output::format_sparql_results(json.as_bytes(), format)?;
        }
        RepoCommand::Create(args) => {
            let config = rdf4j_rs::generate_repo_config(
                &args.id,
                args.title.as_deref(),
                args.repo_type.into(),
            )?;
            client.create_repo(&args.id, config)?;
            println!("Repository '{}' created.", args.id);
        }
        RepoCommand::Delete { id } => {
            client.delete_repo(id)?;
            println!("Repository '{id}' deleted.");
        }
        RepoCommand::Size { id } => {
            let size = client.repo_size(id)?;
            output::format_scalar("size", size, format);
        }
    }
    Ok(())
}
