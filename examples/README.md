# Examples

Shell scripts demonstrating `rdf4j-cli` usage. Each example is self-contained -- it creates its own repositories and cleans up after itself.

## Prerequisites

Start an RDF4J server:

```sh
docker run -d -p 8080:8080 eclipse/rdf4j-workbench
```

## Examples

| Script | Description |
|--------|-------------|
| [complete_workflow.sh](complete_workflow.sh) | Full lifecycle: create repos, insert data, query in all formats, clean up |
| [query_formats.sh](query_formats.sh) | Table, JSON, and CSV output formats side by side |
| [upload_file.sh](upload_file.sh) | Upload a Turtle file and query with PREFIX declarations |
| [statements.sh](statements.sh) | Add statements from file, filter by subject/predicate, delete by pattern |
| [namespaces.sh](namespaces.sh) | Set, get, list, delete, and clear namespace prefixes |

## Running

```sh
# Run any example
./examples/complete_workflow.sh

# Or with cargo
chmod +x examples/*.sh
./examples/upload_file.sh
```

Each script uses `cargo run --quiet --` so you don't need to install the binary first.
