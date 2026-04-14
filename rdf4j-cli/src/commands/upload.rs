use std::fs::File;
use std::io::BufReader;

use anyhow::Result;
use oxrdfio::{RdfFormat, RdfParser, RdfSerializer};

use crate::cli::{RdfFormatArg, UploadArgs};
use rdf4j_rs::Rdf4jClient;

pub fn handle(client: &Rdf4jClient, args: &UploadArgs) -> Result<()> {
    let input_format = RdfFormatArg::resolve(args.rdf_format, &args.file)?;
    let file = File::open(&args.file)?;
    let reader = BufReader::new(file);

    let mut builder = RdfParser::from_format(input_format);
    if let Some(base) = &args.base_uri {
        builder = builder.with_base_iri(base)?;
    }
    let parser = builder.for_reader(reader);

    let mut serializer = RdfSerializer::from_format(RdfFormat::NQuads).for_writer(Vec::new());
    for quad in parser {
        serializer.serialize_quad(quad?.as_ref())?;
    }
    let nquads = serializer.finish()?;

    client.add_statements(
        &args.repo_id,
        nquads,
        RdfFormat::NQuads.media_type(),
        args.context.as_deref(),
        args.base_uri.as_deref(),
    )?;
    println!(
        "Uploaded '{}' to repository '{}'.",
        args.file.display(),
        args.repo_id
    );
    Ok(())
}
