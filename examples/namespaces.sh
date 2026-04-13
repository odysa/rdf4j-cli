#!/usr/bin/env bash
# Namespace management: set, get, list, and clean up namespace prefixes.
#
# Prerequisites:
#   docker run -d -p 8080:8080 eclipse/rdf4j-workbench
#
# Usage:
#   chmod +x examples/namespaces.sh
#   ./examples/namespaces.sh

set -euo pipefail

CLI="cargo run --quiet --"
export RDF4J_SERVER="http://localhost:8080/rdf4j-server"

REPO="ns-example"

echo "=== Create repository ==="
$CLI repo create --id $REPO --repo-type memory

echo ""
echo "=== Set namespaces ==="
$CLI namespace $REPO set ex http://example.org/
$CLI namespace $REPO set foaf http://xmlns.com/foaf/0.1/
$CLI namespace $REPO set schema http://schema.org/
echo "Set 3 namespace prefixes."

echo ""
echo "=== Get a single namespace ==="
echo -n "ex = "
$CLI namespace $REPO get ex

echo ""
echo "=== List all namespaces ==="
$CLI namespace $REPO list

echo ""
echo "=== Delete one namespace ==="
$CLI namespace $REPO delete schema
echo ""
echo "Remaining namespaces:"
$CLI namespace $REPO list

echo ""
echo "=== Clear all namespaces ==="
$CLI namespace $REPO clear

echo ""
echo "=== Clean up ==="
$CLI repo delete $REPO
echo "Done."
