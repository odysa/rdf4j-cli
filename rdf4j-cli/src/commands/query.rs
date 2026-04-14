use anyhow::Result;

use crate::cli::{OutputFormat, QueryArgs};
use crate::output;
use rdf4j_rs::Rdf4jClient;

pub fn handle(client: &Rdf4jClient, args: &QueryArgs, format: OutputFormat) -> Result<()> {
    let sparql = args.input.resolve()?;
    let infer = !args.no_infer;
    let result = client.sparql_query(&args.repo_id, &sparql, infer)?;
    output::format_sparql_results(result.as_bytes(), format)?;
    Ok(())
}
