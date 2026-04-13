#!/usr/bin/env bash
# Demonstrates the three output formats: table, JSON, and CSV.
#
# Prerequisites:
#   docker run -d -p 8080:8080 eclipse/rdf4j-workbench
#
# Usage:
#   chmod +x examples/query_formats.sh
#   ./examples/query_formats.sh

set -euo pipefail

CLI="cargo run --quiet --"
export RDF4J_SERVER="http://localhost:8080/rdf4j-server"

REPO="format-example"

echo "=== Setup ==="
$CLI repo create --id $REPO --repo-type memory
$CLI update $REPO 'INSERT DATA {
  <http://example.org/earth>   <http://example.org/name> "Earth"   . <http://example.org/earth>   <http://example.org/type> "planet" .
  <http://example.org/mars>    <http://example.org/name> "Mars"    . <http://example.org/mars>    <http://example.org/type> "planet" .
  <http://example.org/jupiter> <http://example.org/name> "Jupiter" . <http://example.org/jupiter> <http://example.org/type> "planet" .
  <http://example.org/moon>    <http://example.org/name> "Moon"    . <http://example.org/moon>    <http://example.org/type> "satellite" .
}'
echo "Inserted 8 triples."

QUERY='SELECT ?name ?type WHERE {
  ?body <http://example.org/name> ?name ;
        <http://example.org/type> ?type .
} ORDER BY ?type ?name'

echo ""
echo "=== Table format (default) ==="
$CLI query $REPO "$QUERY"

echo ""
echo "=== JSON format ==="
$CLI --format json query $REPO "$QUERY"

echo ""
echo "=== CSV format ==="
$CLI --format csv query $REPO "$QUERY"

echo ""
echo "=== Clean up ==="
$CLI repo delete $REPO
echo "Done."
