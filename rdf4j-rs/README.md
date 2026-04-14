# rdf4j-rs

Rust SDK for the [Eclipse RDF4J](https://rdf4j.org/) REST API.

[![crates.io](https://img.shields.io/crates/v/rdf4j-rs.svg)](https://crates.io/crates/rdf4j-rs)

## Installation

```toml
[dependencies]
rdf4j-rs = "0.1"
```

## Usage

```rust
use rdf4j_rs::{Rdf4jClient, RepoType, StatementFilter, generate_repo_config};

let client = Rdf4jClient::new("http://localhost:8080/rdf4j-server")?;

// Server
client.health()?;
client.protocol()?;

// Repositories
client.list_repos()?;
let config = generate_repo_config("my-repo", Some("My Repo"), RepoType::Memory)?;
client.create_repo("my-repo", config)?;
client.repo_size("my-repo")?;
client.delete_repo("my-repo")?;

// SPARQL
client.sparql_query("my-repo", "SELECT * WHERE { ?s ?p ?o } LIMIT 10", true)?;
client.sparql_update("my-repo", "INSERT DATA { <urn:a> <urn:b> <urn:c> . }".into())?;

// Statements
let filter = StatementFilter { subj: Some("<urn:a>".into()), ..Default::default() };
client.get_statements("my-repo", &filter, true)?;
client.add_statements("my-repo", body, "text/turtle", None, None)?;
client.delete_statements("my-repo", &filter)?;

// Namespaces
client.list_namespaces("my-repo")?;
client.get_namespace("my-repo", "ex")?;
client.set_namespace("my-repo", "ex", "http://example.org/")?;
client.delete_namespace("my-repo", "ex")?;
client.clear_namespaces("my-repo")?;
```

## API

### `Rdf4jClient`

| Method | Description |
|--------|-------------|
| `new(base_url)` | Create a client for the given RDF4J server URL |
| `health()` | Check if the server is reachable |
| `protocol()` | Get the protocol version |
| `list_repos()` | List all repositories (SPARQL JSON) |
| `create_repo(id, config)` | Create a repository from a Turtle config |
| `delete_repo(id)` | Delete a repository |
| `repo_size(id)` | Get the statement count |
| `sparql_query(repo, query, infer)` | Execute a SPARQL query (SPARQL JSON) |
| `sparql_update(repo, update)` | Execute a SPARQL update |
| `get_statements(repo, filter, infer)` | Get statements as N-Quads |
| `add_statements(repo, body, content_type, context, base_uri)` | Add statements |
| `delete_statements(repo, filter)` | Delete statements by pattern |
| `list_namespaces(repo)` | List namespace prefixes (SPARQL JSON) |
| `get_namespace(repo, prefix)` | Get the URI for a prefix |
| `set_namespace(repo, prefix, uri)` | Set a namespace prefix |
| `delete_namespace(repo, prefix)` | Delete a namespace prefix |
| `clear_namespaces(repo)` | Delete all namespace prefixes |

### `generate_repo_config(id, title, repo_type)`

Generates a Turtle-serialized repository configuration for `create_repo`. Supports `RepoType::Memory` and `RepoType::Native`.

### `StatementFilter`

Optional filters for statement operations: `subj`, `pred`, `obj`, `context`.

## Related

- [rdf4j-cli](https://crates.io/crates/rdf4j-cli) -- CLI built on this library
- [Eclipse RDF4J](https://rdf4j.org/) -- The RDF4J framework

## License

MIT
