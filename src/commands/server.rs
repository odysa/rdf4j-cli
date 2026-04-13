use anyhow::Result;

use crate::cli::{OutputFormat, ServerCommand};
use crate::client::Rdf4jClient;
use crate::output;

pub fn handle(client: &Rdf4jClient, cmd: &ServerCommand, format: OutputFormat) -> Result<()> {
    match cmd {
        ServerCommand::Health => {
            let healthy = client.health()?;
            if healthy {
                output::format_scalar("status", "healthy", format);
            } else {
                output::format_scalar("status", "unreachable", format);
                std::process::exit(1);
            }
        }
        ServerCommand::Protocol => {
            let version = client.protocol()?;
            output::format_scalar("protocol", version.trim(), format);
        }
    }
    Ok(())
}
