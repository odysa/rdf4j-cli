use anyhow::Result;

use crate::cli::UpdateArgs;
use crate::client::Rdf4jClient;

pub fn handle(client: &Rdf4jClient, args: &UpdateArgs) -> Result<()> {
    let sparql = args.input.resolve()?;
    client.sparql_update(&args.repo_id, sparql)?;
    println!("Update executed successfully.");
    Ok(())
}
