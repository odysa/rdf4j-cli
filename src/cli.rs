use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use oxrdfio::RdfFormat;

#[derive(Parser)]
#[command(name = "rdf4j", version, about = "CLI for managing RDF4J repositories")]
pub struct Cli {
    /// RDF4J server URL
    #[arg(
        long,
        global = true,
        env = "RDF4J_SERVER",
        default_value = "http://localhost:8080/rdf4j-server"
    )]
    pub server: String,

    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Server-level operations
    Server(ServerArgs),
    /// Repository management
    Repo(RepoArgs),
    /// Execute a SPARQL query
    Query(QueryArgs),
    /// Execute a SPARQL update
    Update(UpdateArgs),
    /// Namespace management
    Namespace(NamespaceArgs),
    /// Statement management
    Statement(StatementArgs),
    /// Upload an RDF file to a repository
    Upload(UploadArgs),
}

#[derive(Args)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub command: ServerCommand,
}

#[derive(Subcommand)]
pub enum ServerCommand {
    /// Check if the server is reachable
    Health,
    /// Get the RDF4J protocol version
    Protocol,
}

#[derive(Args)]
pub struct RepoArgs {
    #[command(subcommand)]
    pub command: RepoCommand,
}

#[derive(Subcommand)]
pub enum RepoCommand {
    /// List all repositories
    List,
    /// Create a new repository
    Create(RepoCreateArgs),
    /// Delete a repository
    Delete {
        /// Repository ID
        id: String,
    },
    /// Get the number of statements in a repository
    Size {
        /// Repository ID
        id: String,
    },
}

#[derive(Args)]
pub struct RepoCreateArgs {
    /// Repository ID
    #[arg(long)]
    pub id: String,

    /// Human-readable title
    #[arg(long)]
    pub title: Option<String>,

    /// Store type
    #[arg(long, value_enum, default_value_t = RepoType::Memory)]
    pub repo_type: RepoType,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum RepoType {
    Memory,
    Native,
}

/// Shared args for providing a SPARQL string via positional arg or --file.
#[derive(Args)]
pub struct SparqlInput {
    /// SPARQL string (positional)
    pub query: Option<String>,

    /// Read SPARQL from file
    #[arg(long, conflicts_with = "query")]
    pub file: Option<PathBuf>,
}

impl SparqlInput {
    pub fn resolve(&self) -> anyhow::Result<String> {
        if let Some(q) = &self.query {
            Ok(q.clone())
        } else if let Some(f) = &self.file {
            Ok(std::fs::read_to_string(f)?)
        } else {
            anyhow::bail!("No SPARQL provided. Pass it as an argument or use --file.")
        }
    }
}

#[derive(Args)]
pub struct QueryArgs {
    /// Repository ID
    pub repo_id: String,

    #[command(flatten)]
    pub input: SparqlInput,

    /// Disable inference
    #[arg(long)]
    pub no_infer: bool,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Repository ID
    pub repo_id: String,

    #[command(flatten)]
    pub input: SparqlInput,
}

#[derive(Args)]
pub struct NamespaceArgs {
    /// Repository ID
    pub repo_id: String,

    #[command(subcommand)]
    pub command: NamespaceCommand,
}

#[derive(Subcommand)]
pub enum NamespaceCommand {
    /// List all namespaces
    List,
    /// Get a namespace URI by prefix
    Get {
        /// Namespace prefix
        prefix: String,
    },
    /// Set a namespace prefix to a URI
    Set {
        /// Namespace prefix
        prefix: String,
        /// Namespace URI
        uri: String,
    },
    /// Delete a namespace by prefix
    Delete {
        /// Namespace prefix
        prefix: String,
    },
    /// Clear all namespaces
    Clear,
}

/// Shared filter fields for statement get/delete.
#[derive(Args, Default)]
pub struct StatementFilter {
    /// Filter by subject (N-Triples encoded IRI, e.g. <http://example.org/s>)
    #[arg(long)]
    pub subj: Option<String>,
    /// Filter by predicate
    #[arg(long)]
    pub pred: Option<String>,
    /// Filter by object
    #[arg(long)]
    pub obj: Option<String>,
    /// Filter by named graph context
    #[arg(long)]
    pub context: Option<String>,
}

#[derive(Args)]
pub struct StatementArgs {
    /// Repository ID
    pub repo_id: String,

    #[command(subcommand)]
    pub command: StatementCommand,
}

#[derive(Subcommand)]
pub enum StatementCommand {
    /// Get statements matching a pattern
    Get(StatementGetArgs),
    /// Add statements from an RDF file
    Add(StatementAddArgs),
    /// Delete statements matching a pattern
    Delete(StatementFilter),
}

#[derive(Args)]
pub struct StatementGetArgs {
    #[command(flatten)]
    pub filter: StatementFilter,
    /// Disable inference
    #[arg(long)]
    pub no_infer: bool,
}

#[derive(Args)]
pub struct StatementAddArgs {
    /// Path to RDF file containing statements
    #[arg(long)]
    pub file: PathBuf,
    /// RDF format (auto-detected from extension if omitted)
    #[arg(long, value_enum)]
    pub rdf_format: Option<RdfFormatArg>,
}

#[derive(Args)]
pub struct UploadArgs {
    /// Repository ID
    pub repo_id: String,

    /// Path to the RDF file
    pub file: PathBuf,

    /// RDF format (auto-detected from extension if omitted)
    #[arg(long, value_enum)]
    pub rdf_format: Option<RdfFormatArg>,

    /// Target named graph URI
    #[arg(long)]
    pub context: Option<String>,

    /// Base URI for resolving relative URIs
    #[arg(long)]
    pub base_uri: Option<String>,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum RdfFormatArg {
    Turtle,
    Ntriples,
    Nquads,
    Rdfxml,
    Jsonld,
    Trig,
    N3,
}

impl RdfFormatArg {
    pub fn to_rdf_format(self) -> RdfFormat {
        match self {
            Self::Turtle => RdfFormat::Turtle,
            Self::Ntriples => RdfFormat::NTriples,
            Self::Nquads => RdfFormat::NQuads,
            Self::Rdfxml => RdfFormat::RdfXml,
            Self::Jsonld => RdfFormat::JsonLd {
                profile: oxrdfio::JsonLdProfileSet::empty(),
            },
            Self::Trig => RdfFormat::TriG,
            Self::N3 => RdfFormat::N3,
        }
    }

    /// Resolve the RDF format: use the explicit arg if provided, otherwise detect from file extension.
    pub fn resolve(explicit: Option<Self>, path: &std::path::Path) -> anyhow::Result<RdfFormat> {
        match explicit {
            Some(f) => Ok(f.to_rdf_format()),
            None => {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                RdfFormat::from_extension(ext).ok_or_else(|| {
                    anyhow::anyhow!("Could not detect RDF format for file: {}", path.display())
                })
            }
        }
    }
}
