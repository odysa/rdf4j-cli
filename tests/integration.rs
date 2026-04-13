//! Integration tests against a real RDF4J server running in Docker.
//!
//! These tests use testcontainers to spin up an `eclipse/rdf4j-workbench`
//! container. They require Docker to be running.

use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use rand::Rng;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use rdf4j_cli::cli::{RepoType, StatementFilter};
use rdf4j_cli::client::Rdf4jClient;
use rdf4j_cli::commands::repo::generate_repo_config;

const RDF4J_PORT: u16 = 8080;

/// Shared container across all tests to avoid repeated ~10s startup.
static CONTAINER: OnceLock<ContainerState> = OnceLock::new();

struct ContainerState {
    base_url: String,
    _container: ContainerAsync<GenericImage>,
}

async fn get_or_init_container() -> &'static ContainerState {
    if let Some(state) = CONTAINER.get() {
        return state;
    }

    let container = GenericImage::new("eclipse/rdf4j-workbench", "latest")
        .with_exposed_port(RDF4J_PORT.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "org.eclipse.jetty.server.Server - Started",
        ))
        .with_startup_timeout(Duration::from_secs(120))
        .start()
        .await
        .expect("Failed to start RDF4J container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(RDF4J_PORT).await.unwrap();
    let base_url = format!("http://{host}:{port}/rdf4j-server");

    wait_for_server(&base_url);

    let state = ContainerState {
        base_url,
        _container: container,
    };

    let _ = CONTAINER.set(state);
    CONTAINER.get().unwrap()
}

fn wait_for_server(base_url: &str) {
    let client = Rdf4jClient::new(base_url).expect("Failed to create client");
    for _ in 0..300 {
        if client.health().unwrap_or(false) {
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }
    panic!("RDF4J server did not become ready within 150 seconds");
}

fn new_client(base_url: &str) -> Rdf4jClient {
    Rdf4jClient::new(base_url).expect("Failed to create client")
}

fn random_repo_id() -> String {
    let n: u32 = rand::rng().random_range(1..1_000_000);
    format!("test_repo_{n}")
}

fn test_repo_config(id: &str) -> Vec<u8> {
    generate_repo_config(id, None, RepoType::Memory).expect("Failed to generate config")
}

// ── Server tests ────────────────────────────────────────

#[tokio::test]
async fn test_health() {
    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    assert!(client.health().unwrap());
}

#[tokio::test]
async fn test_protocol() {
    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    let version = client.protocol().unwrap();
    assert!(!version.is_empty());
}

// ── Repository tests ───────────────────────────────────

#[tokio::test]
async fn test_create_list_delete_repo() {
    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    let repo_id = random_repo_id();

    client
        .create_repo(&repo_id, test_repo_config(&repo_id))
        .unwrap();

    let repos_json = client.list_repos().unwrap();
    assert!(repos_json.contains(&repo_id));

    let size = client.repo_size(&repo_id).unwrap();
    assert_eq!(size, 0);

    client.delete_repo(&repo_id).unwrap();

    let repos_json = client.list_repos().unwrap();
    assert!(!repos_json.contains(&repo_id));
}

// ── SPARQL query/update tests ──────────────────────────

#[tokio::test]
async fn test_sparql_insert_and_query() {
    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    let repo_id = random_repo_id();

    client
        .create_repo(&repo_id, test_repo_config(&repo_id))
        .unwrap();

    let insert = r#"
        INSERT DATA {
            <http://example.org/alice> <http://example.org/name> "Alice" .
            <http://example.org/bob> <http://example.org/name> "Bob" .
        }
    "#;
    client
        .sparql_update(&repo_id, insert.to_string())
        .unwrap();

    assert_eq!(client.repo_size(&repo_id).unwrap(), 2);

    let result = client
        .sparql_query(
            &repo_id,
            "SELECT ?s ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name",
            true,
        )
        .unwrap();
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));

    let ask_result = client
        .sparql_query(
            &repo_id,
            r#"ASK { <http://example.org/alice> <http://example.org/name> "Alice" }"#,
            true,
        )
        .unwrap();
    assert!(ask_result.contains("true"));

    client.delete_repo(&repo_id).unwrap();
}

// ── Statement tests ─────────────────────────────────────

#[tokio::test]
async fn test_add_get_delete_statements() {
    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    let repo_id = random_repo_id();

    client
        .create_repo(&repo_id, test_repo_config(&repo_id))
        .unwrap();

    let nquads = b"<http://example.org/s1> <http://example.org/p1> \"value1\" .\n\
                   <http://example.org/s2> <http://example.org/p2> \"value2\" .\n";
    client
        .add_statements(
            &repo_id,
            nquads.to_vec(),
            "application/n-triples",
            None,
            None,
        )
        .unwrap();

    assert_eq!(client.repo_size(&repo_id).unwrap(), 2);

    let stmts = client
        .get_statements(&repo_id, &StatementFilter::default(), true)
        .unwrap();
    assert!(stmts.contains("example.org/s1"));
    assert!(stmts.contains("example.org/s2"));

    let subj_filter = StatementFilter {
        subj: Some("<http://example.org/s1>".to_string()),
        ..StatementFilter::default()
    };
    let stmts = client
        .get_statements(&repo_id, &subj_filter, true)
        .unwrap();
    assert!(stmts.contains("example.org/s1"));
    assert!(!stmts.contains("example.org/s2"));

    client.delete_statements(&repo_id, &subj_filter).unwrap();
    assert_eq!(client.repo_size(&repo_id).unwrap(), 1);

    client.delete_repo(&repo_id).unwrap();
}

// ── Namespace tests ─────────────────────────────────────

#[tokio::test]
async fn test_namespace_crud() {
    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    let repo_id = random_repo_id();

    client
        .create_repo(&repo_id, test_repo_config(&repo_id))
        .unwrap();

    client
        .set_namespace(&repo_id, "ex", "http://example.org/")
        .unwrap();

    let ns = client.get_namespace(&repo_id, "ex").unwrap();
    assert_eq!(ns.trim(), "http://example.org/");

    let all_ns = client.list_namespaces(&repo_id).unwrap();
    assert!(all_ns.contains("ex"));

    client.delete_namespace(&repo_id, "ex").unwrap();

    let result = client.get_namespace(&repo_id, "ex");
    assert!(result.is_err());

    client.clear_namespaces(&repo_id).unwrap();

    client.delete_repo(&repo_id).unwrap();
}

// ── Upload tests ────────────────────────────────────────

#[tokio::test]
async fn test_upload_turtle_file() {
    use oxrdfio::{RdfFormat, RdfParser, RdfSerializer};

    let state = get_or_init_container().await;
    let client = new_client(&state.base_url);
    let repo_id = random_repo_id();

    client
        .create_repo(&repo_id, test_repo_config(&repo_id))
        .unwrap();

    let turtle = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:name "Alice" ;
                 ex:age "30" .
        ex:bob   ex:name "Bob" .
    "#;

    let mut serializer = RdfSerializer::from_format(RdfFormat::NQuads).for_writer(Vec::new());
    for quad in RdfParser::from_format(RdfFormat::Turtle).for_reader(turtle.as_bytes()) {
        serializer.serialize_quad(quad.unwrap().as_ref()).unwrap();
    }
    let nquads = serializer.finish().unwrap();

    client
        .add_statements(&repo_id, nquads, "application/n-quads", None, None)
        .unwrap();

    assert_eq!(client.repo_size(&repo_id).unwrap(), 3);

    let result = client
        .sparql_query(
            &repo_id,
            r#"SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name"#,
            true,
        )
        .unwrap();
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));

    client.delete_repo(&repo_id).unwrap();
}

// ── Output formatting tests (no Docker needed) ─────────

#[test]
fn test_format_sparql_results_table() {
    use rdf4j_cli::cli::OutputFormat;
    use rdf4j_cli::output;

    let json = r#"{
        "head": {"vars": ["name"]},
        "results": {"bindings": [
            {"name": {"type": "literal", "value": "Alice"}},
            {"name": {"type": "literal", "value": "Bob"}}
        ]}
    }"#;

    output::format_sparql_results(json.as_bytes(), OutputFormat::Table).unwrap();
    output::format_sparql_results(json.as_bytes(), OutputFormat::Json).unwrap();
    output::format_sparql_results(json.as_bytes(), OutputFormat::Csv).unwrap();
}

#[test]
fn test_format_sparql_boolean() {
    use rdf4j_cli::cli::OutputFormat;
    use rdf4j_cli::output;

    let json = r#"{"boolean": true}"#;
    output::format_sparql_results(json.as_bytes(), OutputFormat::Table).unwrap();
    output::format_sparql_results(json.as_bytes(), OutputFormat::Json).unwrap();
}

#[test]
fn test_rdf_format_detect() {
    use rdf4j_cli::cli::RdfFormatArg;
    use std::path::Path;

    // Explicit arg always wins
    assert!(
        RdfFormatArg::resolve(Some(RdfFormatArg::Turtle), Path::new("data.nq")).is_ok()
    );

    // Detection from extension
    assert!(RdfFormatArg::resolve(None, Path::new("data.ttl")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.nt")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.nq")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.rdf")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.jsonld")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.trig")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.n3")).is_ok());
    // .txt maps to N-Triples in oxrdfio, unknown extensions fail
    assert!(RdfFormatArg::resolve(None, Path::new("data.txt")).is_ok());
    assert!(RdfFormatArg::resolve(None, Path::new("data.xyz")).is_err());
}

#[test]
fn test_sparql_input_resolve() {
    use rdf4j_cli::cli::SparqlInput;

    let input = SparqlInput {
        query: Some("SELECT * WHERE { ?s ?p ?o }".to_string()),
        file: None,
    };
    assert_eq!(input.resolve().unwrap(), "SELECT * WHERE { ?s ?p ?o }");

    let input = SparqlInput {
        query: None,
        file: None,
    };
    assert!(input.resolve().is_err());
}
