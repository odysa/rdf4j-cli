use oxrdf::{BlankNode, GraphName, Literal, NamedNode, Quad};
use oxrdfio::{RdfFormat, RdfSerializer};

use crate::error::Rdf4jError;

#[derive(Copy, Clone)]
pub enum RepoType {
    Memory,
    Native,
}

const CONFIG_NS: &str = "tag:rdf4j.org,2023:config/";

fn ns(suffix: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("{CONFIG_NS}{suffix}"))
}

fn emit(
    ser: &mut oxrdfio::WriterQuadSerializer<Vec<u8>>,
    subject: impl Into<oxrdf::NamedOrBlankNode>,
    predicate: NamedNode,
    object: impl Into<oxrdf::Term>,
) -> Result<(), Rdf4jError> {
    let q = Quad::new(subject, predicate, object, GraphName::DefaultGraph);
    ser.serialize_quad(q.as_ref())?;
    Ok(())
}

pub fn generate_repo_config(
    id: &str,
    title: Option<&str>,
    repo_type: RepoType,
) -> Result<Vec<u8>, Rdf4jError> {
    let rdf_type = NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    let rdfs_label = NamedNode::new_unchecked("http://www.w3.org/2000/01/rdf-schema#label");

    let repo_node = BlankNode::default();
    let impl_node = BlankNode::default();
    let sail_node = BlankNode::default();

    let mut ser = RdfSerializer::from_format(RdfFormat::Turtle).for_writer(Vec::new());

    emit(&mut ser, repo_node.clone(), rdf_type, ns("Repository"))?;
    emit(
        &mut ser,
        repo_node.clone(),
        ns("rep.id"),
        Literal::new_simple_literal(id),
    )?;
    if let Some(t) = title {
        emit(
            &mut ser,
            repo_node.clone(),
            rdfs_label,
            Literal::new_simple_literal(t),
        )?;
    }
    emit(
        &mut ser,
        impl_node.clone(),
        ns("rep.type"),
        Literal::new_simple_literal("openrdf:SailRepository"),
    )?;
    emit(&mut ser, repo_node, ns("rep.impl"), impl_node.clone())?;

    let sail_type_value = match repo_type {
        RepoType::Memory => "openrdf:MemoryStore",
        RepoType::Native => "openrdf:NativeStore",
    };
    emit(
        &mut ser,
        sail_node.clone(),
        ns("sail.type"),
        Literal::new_simple_literal(sail_type_value),
    )?;
    emit(&mut ser, impl_node, ns("sail.impl"), sail_node)?;

    Ok(ser.finish()?)
}
