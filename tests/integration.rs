//! Integration tests against a real RDF4J server.
//!
//! Set `RDF4J_TEST_URL` to use an external server (e.g. in CI with a service container).
//! Otherwise, testcontainers spins up `eclipse/rdf4j-workbench` automatically.

use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use rand::Rng;

use rdf4j_cli::cli::{RepoType, StatementFilter};
use rdf4j_cli::client::Rdf4jClient;
use rdf4j_cli::commands::repo::generate_repo_config;

/// Returns the base URL of the RDF4J server, starting a container if needed.
/// All initialization happens on a dedicated thread to avoid nesting tokio runtimes
/// (reqwest::blocking::Client creates its own runtime internally).
fn server_url() -> &'static str {
    static URL: OnceLock<String> = OnceLock::new();

    URL.get_or_init(|| {
        // If RDF4J_TEST_URL is set, use that directly (CI with service container).
        if let Ok(url) = std::env::var("RDF4J_TEST_URL") {
            return url;
        }

        // Spin up a container on a dedicated thread with its own tokio runtime.
        // The container handle is leaked intentionally to keep it alive for the
        // lifetime of the test process.
        // Spin up a container on a dedicated thread with its own tokio runtime.
        // Both the runtime and container are leaked so the container stays alive
        // for the entire test process.
        let url = thread::spawn(|| {
            use testcontainers::core::{IntoContainerPort, WaitFor};
            use testcontainers::runners::AsyncRunner;
            use testcontainers::{GenericImage, ImageExt};

            let rt = tokio::runtime::Runtime::new().unwrap();
            let (url, container) = rt.block_on(async {
                let container = GenericImage::new("eclipse/rdf4j-workbench", "latest")
                    .with_exposed_port(8080.tcp())
                    .with_wait_for(WaitFor::message_on_stderr(
                        "org.eclipse.jetty.server.Server - Started",
                    ))
                    .with_startup_timeout(Duration::from_secs(120))
                    .start()
                    .await
                    .expect("Failed to start RDF4J container");

                let host = container.get_host().await.unwrap();
                let port = container.get_host_port_ipv4(8080).await.unwrap();
                let url = format!("http://{host}:{port}/rdf4j-server");

                (url, container)
            });

            // Leak so they live until process exit.
            std::mem::forget(container);
            std::mem::forget(rt);

            url
        })
        .join()
        .expect("Container thread panicked");

        wait_for_server(&url);
        url
    })
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

fn new_client() -> Rdf4jClient {
    Rdf4jClient::new(server_url()).expect("Failed to create client")
}

fn random_repo_id() -> String {
    let n: u32 = rand::rng().random_range(1..1_000_000);
    format!("test_repo_{n}")
}

fn test_repo_config(id: &str) -> Vec<u8> {
    generate_repo_config(id, None, RepoType::Memory).expect("Failed to generate config")
}

// ── Server tests ────────────────────────────────────────

#[test]
fn test_health() {
    let client = new_client();
    assert!(client.health().unwrap());
}

#[test]
fn test_protocol() {
    let client = new_client();
    let version = client.protocol().unwrap();
    assert!(!version.is_empty());
}

// ── Repository tests ───────────────────────────────────

#[test]
fn test_create_list_delete_repo() {
    let client = new_client();
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

#[test]
fn test_sparql_insert_and_query() {
    let client = new_client();
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
    client.sparql_update(&repo_id, insert.to_string()).unwrap();

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

#[test]
fn test_add_get_delete_statements() {
    let client = new_client();
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
    let stmts = client.get_statements(&repo_id, &subj_filter, true).unwrap();
    assert!(stmts.contains("example.org/s1"));
    assert!(!stmts.contains("example.org/s2"));

    client.delete_statements(&repo_id, &subj_filter).unwrap();
    assert_eq!(client.repo_size(&repo_id).unwrap(), 1);

    client.delete_repo(&repo_id).unwrap();
}

// ── Namespace tests ─────────────────────────────────────

#[test]
fn test_namespace_crud() {
    let client = new_client();
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

#[test]
fn test_upload_turtle_file() {
    use oxrdfio::{RdfFormat, RdfParser, RdfSerializer};

    let client = new_client();
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

// ── E2E test: full lifecycle via the CLI binary ─────────

#[test]
fn test_e2e_create_insert_query_delete() {
    use std::process::Command;

    let url = server_url();
    let bin = env!("CARGO_BIN_EXE_rdf4j-cli");
    let repo_id = random_repo_id();

    let run = |args: &[&str]| -> std::process::Output {
        let output = Command::new(bin)
            .args(["--server", url])
            .args(args)
            .output()
            .expect("Failed to execute CLI");
        if !output.status.success() {
            panic!(
                "CLI failed: {:?}\nstdout: {}\nstderr: {}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        output
    };

    let run_stdout = |args: &[&str]| -> String {
        let output = run(args);
        String::from_utf8(output.stdout).unwrap()
    };

    // 1. Create repository
    let out = run_stdout(&["repo", "create", "--id", &repo_id, "--repo-type", "memory"]);
    assert!(out.contains("created"));

    // 2. Verify it shows up in repo list
    let out = run_stdout(&["repo", "list"]);
    assert!(out.contains(&repo_id));

    // 3. Size should be 0
    let out = run_stdout(&["repo", "size", &repo_id]);
    assert!(out.contains('0'));

    // 4. Insert data via SPARQL UPDATE
    let insert = r#"INSERT DATA {
        <http://example.org/alice> <http://example.org/name> "Alice" .
        <http://example.org/bob> <http://example.org/name> "Bob" .
        <http://example.org/alice> <http://example.org/age> "30" .
    }"#;
    let out = run_stdout(&["update", &repo_id, insert]);
    assert!(out.contains("successfully"));

    // 5. Size should be 3
    let out = run_stdout(&["repo", "size", &repo_id]);
    assert!(out.contains('3'));

    // 6. Query — table format (default)
    let out = run_stdout(&[
        "query",
        &repo_id,
        "SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name",
    ]);
    assert!(out.contains("Alice"));
    assert!(out.contains("Bob"));

    // 7. Query — JSON format
    let out = run_stdout(&[
        "--format",
        "json",
        "query",
        &repo_id,
        "SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name",
    ]);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("Invalid JSON output");
    let names: Vec<&str> = parsed
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    assert!(names.iter().any(|n| n.contains("Alice")));
    assert!(names.iter().any(|n| n.contains("Bob")));

    // 8. Query — CSV format
    let out = run_stdout(&[
        "--format",
        "csv",
        "query",
        &repo_id,
        "SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name",
    ]);
    assert!(out.contains("name")); // header
    assert!(out.contains("Alice"));

    // 9. ASK query
    let out = run_stdout(&[
        "query",
        &repo_id,
        r#"ASK { <http://example.org/alice> <http://example.org/name> "Alice" }"#,
    ]);
    assert!(out.contains("true"));

    // 10. Delete the repository
    let out = run_stdout(&["repo", "delete", &repo_id]);
    assert!(out.contains("deleted"));

    // 11. Verify it's gone
    let out = run_stdout(&["repo", "list"]);
    assert!(!out.contains(&repo_id));
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
    assert!(RdfFormatArg::resolve(Some(RdfFormatArg::Turtle), Path::new("data.nq")).is_ok());

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
