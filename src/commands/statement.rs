use std::fs::File;

use anyhow::Result;

use crate::cli::{OutputFormat, RdfFormatArg, StatementCommand};
use crate::client::Rdf4jClient;
use crate::output;

pub fn handle(
    client: &Rdf4jClient,
    repo_id: &str,
    cmd: &StatementCommand,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        StatementCommand::Get(args) => {
            let infer = !args.no_infer;
            let result = client.get_statements(repo_id, &args.filter, infer)?;
            output::format_raw(&result, format);
        }
        StatementCommand::Add(args) => {
            let rdf_fmt = RdfFormatArg::resolve(args.rdf_format, &args.file)?;
            let body = File::open(&args.file)?;
            client.add_statements(repo_id, body, rdf_fmt.media_type(), None, None)?;
            println!("Statements added from '{}'.", args.file.display());
        }
        StatementCommand::Delete(filter) => {
            client.delete_statements(repo_id, filter)?;
            println!("Statements deleted.");
        }
    }
    Ok(())
}
