# rdf4j-cli

A command-line tool for managing [Eclipse RDF4J](https://rdf4j.org/) repositories via the REST API.

[![CI](https://github.com/odysa/rdf4j-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/odysa/rdf4j-cli/actions/workflows/ci.yml)

## Features

- **Repository management** -- create, list, delete, and inspect repositories
- **SPARQL queries** -- SELECT, ASK, CONSTRUCT, DESCRIBE with table, JSON, or CSV output
- **SPARQL updates** -- INSERT, DELETE, and other update operations
- **Statement operations** -- get, add, and delete RDF statements with pattern filters
- **Namespace management** -- list, get, set, delete, and clear namespace prefixes
- **File upload** -- upload RDF files (Turtle, N-Triples, N-Quads, RDF/XML, JSON-LD, TriG, N3) with automatic format detection
- **Multiple output formats** -- `--format table` (default), `json`, or `csv`

## Installation

### From source

```sh
cargo install --path .
```

### Build from Git

```sh
git clone https://github.com/odysa/rdf4j-cli.git
cd rdf4j-cli
cargo build --release
# Binary at target/release/rdf4j-cli
```

## Quick start

Start an RDF4J server (e.g. with Docker):

```sh
docker run -d -p 8080:8080 eclipse/rdf4j-workbench
```

Then:

```sh
# Create a repository
rdf4j-cli repo create --id my-repo --repo-type memory

# Insert data
rdf4j-cli update my-repo 'INSERT DATA {
  <http://example.org/alice> <http://example.org/name> "Alice" .
  <http://example.org/bob>   <http://example.org/name> "Bob" .
}'

# Query
rdf4j-cli query my-repo 'SELECT ?s ?name WHERE { ?s <http://example.org/name> ?name }'

# Output:
# ╭──────────────────────────────┬─────────╮
# │ s                            │ name    │
# ├──────────────────────────────┼─────────┤
# │ <http://example.org/alice>   │ "Alice" │
# │ <http://example.org/bob>     │ "Bob"   │
# ╰──────────────────────────────┴─────────╯

# Delete the repository
rdf4j-cli repo delete my-repo
```

## Usage

### Global options

```
--server <URL>    RDF4J server URL (or set RDF4J_SERVER env var)
                  [default: http://localhost:8080/rdf4j-server]
--format <FMT>    Output format: table, json, csv [default: table]
```

### Server

```sh
rdf4j-cli server health       # Check if server is reachable
rdf4j-cli server protocol     # Get protocol version
```

### Repositories

```sh
rdf4j-cli repo list                                    # List all repositories
rdf4j-cli repo create --id my-repo --repo-type memory  # Create (memory or native)
rdf4j-cli repo create --id my-repo --title "My Repo"   # With a title
rdf4j-cli repo size my-repo                            # Statement count
rdf4j-cli repo delete my-repo                          # Delete
```

### SPARQL query

```sh
# Inline query
rdf4j-cli query my-repo 'SELECT * WHERE { ?s ?p ?o } LIMIT 10'

# From file
rdf4j-cli query my-repo --file query.rq

# JSON output
rdf4j-cli --format json query my-repo 'SELECT * WHERE { ?s ?p ?o }'

# Disable inference
rdf4j-cli query my-repo --no-infer 'SELECT * WHERE { ?s ?p ?o }'
```

### SPARQL update

```sh
# Inline update
rdf4j-cli update my-repo 'INSERT DATA { <http://example.org/s> <http://example.org/p> "value" . }'

# From file
rdf4j-cli update my-repo --file update.ru
```

### Namespaces

```sh
rdf4j-cli namespace my-repo list                                 # List all
rdf4j-cli namespace my-repo get ex                               # Get URI for prefix
rdf4j-cli namespace my-repo set ex http://example.org/           # Set prefix
rdf4j-cli namespace my-repo delete ex                            # Delete prefix
rdf4j-cli namespace my-repo clear                                # Clear all
```

### Statements

```sh
# Get all statements
rdf4j-cli statement my-repo get

# Filter by subject
rdf4j-cli statement my-repo get --subj '<http://example.org/alice>'

# Add from file
rdf4j-cli statement my-repo add --file data.nt

# Delete by pattern
rdf4j-cli statement my-repo delete --subj '<http://example.org/alice>'
```

### File upload

```sh
# Auto-detect format from extension
rdf4j-cli upload my-repo data.ttl

# Explicit format
rdf4j-cli upload my-repo data.txt --rdf-format ntriples

# Upload into a named graph
rdf4j-cli upload my-repo data.ttl --context http://example.org/graph

# With base URI
rdf4j-cli upload my-repo data.ttl --base-uri http://example.org/
```

Supported formats: Turtle (`.ttl`), N-Triples (`.nt`), N-Quads (`.nq`), RDF/XML (`.rdf`, `.xml`), JSON-LD (`.jsonld`), TriG (`.trig`), N3 (`.n3`).

## Configuration

The server URL can be set three ways (in order of precedence):

1. `--server` flag
2. `RDF4J_SERVER` environment variable
3. Default: `http://localhost:8080/rdf4j-server`

```sh
# Using env var
export RDF4J_SERVER=http://my-server:8080/rdf4j-server
rdf4j-cli repo list

# Using flag
rdf4j-cli --server http://my-server:8080/rdf4j-server repo list
```

## Development

### Build

```sh
cargo build
```

### Lint

```sh
cargo clippy --all-targets
```

### Tests

Unit tests (no Docker required):

```sh
cargo test -- --skip test_health --skip test_protocol --skip test_create \
  --skip test_sparql --skip test_add --skip test_namespace --skip test_upload --skip test_e2e
```

Integration + e2e tests (requires Docker):

```sh
cargo test --test integration -- --test-threads=1
```

Or with an external RDF4J server:

```sh
RDF4J_TEST_URL=http://localhost:8080/rdf4j-server cargo test --test integration -- --test-threads=1
```

## Examples

See the [examples/](examples/) folder for runnable shell scripts:

- **[complete_workflow.sh](examples/complete_workflow.sh)** -- Full lifecycle: create, insert, query (table/JSON/CSV), delete
- **[query_formats.sh](examples/query_formats.sh)** -- Table, JSON, and CSV output side by side
- **[upload_file.sh](examples/upload_file.sh)** -- Upload Turtle files and query with PREFIX
- **[statements.sh](examples/statements.sh)** -- Add, filter, and delete statements
- **[namespaces.sh](examples/namespaces.sh)** -- Namespace prefix management

## Architecture

```
src/
  main.rs           Entry point
  lib.rs            Public module exports
  cli.rs            Clap command definitions
  client.rs         RDF4J HTTP client
  error.rs          Error types
  output.rs         Table/JSON/CSV formatting
  commands/
    server.rs       health, protocol
    repo.rs         list, create, delete, size
    query.rs        SPARQL query
    update.rs       SPARQL update
    namespace.rs    namespace CRUD
    statement.rs    statement get/add/delete
    upload.rs       RDF file upload
```

### Key dependencies

- [clap](https://crates.io/crates/clap) -- CLI argument parsing
- [reqwest](https://crates.io/crates/reqwest) -- HTTP client (blocking)
- [oxrdf](https://crates.io/crates/oxrdf) + [oxrdfio](https://crates.io/crates/oxrdfio) -- RDF types and serialization
- [sparesults](https://crates.io/crates/sparesults) -- SPARQL results parsing
- [tabled](https://crates.io/crates/tabled) -- Table output formatting

## Related

- [rdf4j-python](https://github.com/odysa/rdf4j-python) -- Python client library for RDF4J (the reference implementation this CLI is based on)
- [Eclipse RDF4J](https://rdf4j.org/) -- The RDF4J framework

## License

MIT
