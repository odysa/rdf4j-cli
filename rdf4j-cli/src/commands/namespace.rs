use anyhow::Result;

use crate::cli::{NamespaceCommand, OutputFormat};
use crate::output;
use rdf4j_rs::Rdf4jClient;

pub fn handle(
    client: &Rdf4jClient,
    repo_id: &str,
    cmd: &NamespaceCommand,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        NamespaceCommand::List => {
            let json = client.list_namespaces(repo_id)?;
            output::format_sparql_results(json.as_bytes(), format)?;
        }
        NamespaceCommand::Get { prefix } => {
            let uri = client.get_namespace(repo_id, prefix)?;
            output::format_scalar("namespace", uri.trim(), format);
        }
        NamespaceCommand::Set { prefix, uri } => {
            client.set_namespace(repo_id, prefix, uri)?;
            println!("Namespace '{prefix}' set to '{uri}'.");
        }
        NamespaceCommand::Delete { prefix } => {
            client.delete_namespace(repo_id, prefix)?;
            println!("Namespace '{prefix}' deleted.");
        }
        NamespaceCommand::Clear => {
            client.clear_namespaces(repo_id)?;
            println!("All namespaces cleared.");
        }
    }
    Ok(())
}
