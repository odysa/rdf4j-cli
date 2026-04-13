#!/usr/bin/env bash
# Upload an RDF file and query the results.
#
# Prerequisites:
#   docker run -d -p 8080:8080 eclipse/rdf4j-workbench
#
# Usage:
#   chmod +x examples/upload_file.sh
#   ./examples/upload_file.sh

set -euo pipefail

CLI="cargo run --quiet --"
export RDF4J_SERVER="http://localhost:8080/rdf4j-server"

REPO="upload-example"

echo "=== Create repository ==="
$CLI repo create --id $REPO --repo-type memory

echo ""
echo "=== Create sample Turtle file ==="
cat > /tmp/sample_data.ttl << 'TTL'
@prefix ex: <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:alice a foaf:Person ;
    foaf:name "Alice" ;
    foaf:age "30"^^xsd:integer ;
    foaf:knows ex:bob .

ex:bob a foaf:Person ;
    foaf:name "Bob" ;
    foaf:age "25"^^xsd:integer ;
    foaf:knows ex:carol .

ex:carol a foaf:Person ;
    foaf:name "Carol" ;
    foaf:age "28"^^xsd:integer .
TTL
echo "Created /tmp/sample_data.ttl"

echo ""
echo "=== Upload file ==="
$CLI upload $REPO /tmp/sample_data.ttl

echo ""
echo "=== Check size ==="
echo -n "Statements: "
$CLI repo size $REPO

echo ""
echo "=== Query: all people and their ages ==="
$CLI query $REPO \
  'PREFIX foaf: <http://xmlns.com/foaf/0.1/>
   SELECT ?name ?age WHERE {
     ?person a foaf:Person ;
             foaf:name ?name ;
             foaf:age ?age .
   } ORDER BY ?name'

echo ""
echo "=== Query: who knows whom? ==="
$CLI query $REPO \
  'PREFIX foaf: <http://xmlns.com/foaf/0.1/>
   SELECT ?person ?friend WHERE {
     ?p foaf:name ?person ;
        foaf:knows ?f .
     ?f foaf:name ?friend .
   } ORDER BY ?person'

echo ""
echo "=== Clean up ==="
rm -f /tmp/sample_data.ttl
$CLI repo delete $REPO
echo "Done."
