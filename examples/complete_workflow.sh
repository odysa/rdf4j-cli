#!/usr/bin/env bash
# Complete RDF4J workflow: create, populate, query, and clean up repositories.
#
# Prerequisites:
#   docker run -d -p 8080:8080 eclipse/rdf4j-workbench
#
# Usage:
#   chmod +x examples/complete_workflow.sh
#   ./examples/complete_workflow.sh

set -euo pipefail

CLI="cargo run --quiet --"
export RDF4J_SERVER="http://localhost:8080/rdf4j-server"

echo "=== Step 1: Check server health ==="
$CLI server health

echo ""
echo "=== Step 2: Create repositories ==="
$CLI repo create --id customer-data --title "Customer Data" --repo-type memory
$CLI repo create --id product-catalog --title "Product Catalog" --repo-type memory
echo "Created 2 repositories."

echo ""
echo "=== Step 3: List repositories ==="
$CLI repo list

echo ""
echo "=== Step 4: Insert customer data ==="
$CLI update customer-data 'INSERT DATA {
  <http://example.com/customer/1> <http://example.com/name> "Alice Johnson" .
  <http://example.com/customer/1> <http://example.com/email> "alice@example.com" .
  <http://example.com/customer/2> <http://example.com/name> "Bob Smith" .
  <http://example.com/customer/2> <http://example.com/email> "bob@example.com" .
  <http://example.com/customer/3> <http://example.com/name> "Carol White" .
  <http://example.com/customer/3> <http://example.com/email> "carol@example.com" .
}'

echo ""
echo "=== Step 5: Insert product data ==="
$CLI update product-catalog 'INSERT DATA {
  <http://example.com/product/laptop> <http://example.com/name> "Professional Laptop" .
  <http://example.com/product/laptop> <http://example.com/price> "1299.99" .
  <http://example.com/product/laptop> <http://example.com/category> "Electronics" .
  <http://example.com/product/phone>  <http://example.com/name> "Smartphone Pro" .
  <http://example.com/product/phone>  <http://example.com/price> "899.99" .
  <http://example.com/product/phone>  <http://example.com/category> "Electronics" .
}'

echo ""
echo "=== Step 6: Check repository sizes ==="
echo -n "customer-data: "
$CLI repo size customer-data
echo -n "product-catalog: "
$CLI repo size product-catalog

echo ""
echo "=== Step 7: Query customers (table) ==="
$CLI query customer-data \
  'SELECT ?customer ?name ?email WHERE {
    ?customer <http://example.com/name> ?name .
    ?customer <http://example.com/email> ?email .
  } ORDER BY ?name'

echo ""
echo "=== Step 8: Query products (JSON) ==="
$CLI --format json query product-catalog \
  'SELECT ?name ?price WHERE {
    ?product <http://example.com/name> ?name .
    ?product <http://example.com/price> ?price .
  } ORDER BY ?price'

echo ""
echo "=== Step 9: Query products (CSV) ==="
$CLI --format csv query product-catalog \
  'SELECT ?name ?price ?category WHERE {
    ?product <http://example.com/name> ?name .
    ?product <http://example.com/price> ?price .
    ?product <http://example.com/category> ?category .
  } ORDER BY ?name'

echo ""
echo "=== Step 10: ASK query ==="
$CLI query customer-data \
  'ASK { <http://example.com/customer/1> <http://example.com/name> "Alice Johnson" }'

echo ""
echo "=== Step 11: Clean up ==="
$CLI repo delete customer-data
$CLI repo delete product-catalog
echo "Deleted all repositories."

echo ""
echo "=== Done ==="
