#!/usr/bin/env bash
# Statement operations: add from file, get with filters, and delete by pattern.
#
# Prerequisites:
#   docker run -d -p 8080:8080 eclipse/rdf4j-workbench
#
# Usage:
#   chmod +x examples/statements.sh
#   ./examples/statements.sh

set -euo pipefail

CLI="cargo run --quiet --"
export RDF4J_SERVER="http://localhost:8080/rdf4j-server"

REPO="stmt-example"

echo "=== Create repository ==="
$CLI repo create --id $REPO --repo-type memory

echo ""
echo "=== Create N-Triples file ==="
cat > /tmp/sample_statements.nt << 'NT'
<http://example.org/alice> <http://example.org/name> "Alice" .
<http://example.org/alice> <http://example.org/role> "engineer" .
<http://example.org/bob> <http://example.org/name> "Bob" .
<http://example.org/bob> <http://example.org/role> "designer" .
<http://example.org/carol> <http://example.org/name> "Carol" .
<http://example.org/carol> <http://example.org/role> "manager" .
NT
echo "Created /tmp/sample_statements.nt"

echo ""
echo "=== Add statements from file ==="
$CLI statement $REPO add --file /tmp/sample_statements.nt

echo ""
echo "=== Get all statements ==="
$CLI statement $REPO get

echo ""
echo "=== Get statements for alice only ==="
$CLI statement $REPO get --subj '<http://example.org/alice>'

echo ""
echo "=== Get all names (filter by predicate) ==="
$CLI statement $REPO get --pred '<http://example.org/name>'

echo ""
echo "=== Delete bob's statements ==="
$CLI statement $REPO delete --subj '<http://example.org/bob>'

echo ""
echo "=== Remaining statements ==="
$CLI statement $REPO get

echo ""
echo "=== Clean up ==="
rm -f /tmp/sample_statements.nt
$CLI repo delete $REPO
echo "Done."
