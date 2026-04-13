use anyhow::{Context, Result};
use sparesults::{QueryResultsFormat, QueryResultsParser, ReaderQueryResultsParserOutput};
use tabled::builder::Builder;
use tabled::settings::Style;

use crate::cli::OutputFormat;

/// Format SPARQL results JSON (application/sparql-results+json) for display.
pub fn format_sparql_results(json_bytes: &[u8], format: OutputFormat) -> Result<()> {
    let parser = QueryResultsParser::from_format(QueryResultsFormat::Json);
    let parsed = parser
        .for_reader(json_bytes)
        .context("Failed to parse SPARQL results")?;

    match parsed {
        ReaderQueryResultsParserOutput::Boolean(value) => {
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({ "boolean": value }));
                }
                _ => {
                    println!("{value}");
                }
            }
        }
        ReaderQueryResultsParserOutput::Solutions(solutions) => {
            let vars: Vec<String> = solutions
                .variables()
                .iter()
                .map(|v| v.as_str().to_string())
                .collect();

            let mut rows: Vec<Vec<String>> = Vec::new();
            for solution in solutions {
                let solution = solution.context("Failed to read SPARQL solution")?;
                let row: Vec<String> = vars
                    .iter()
                    .map(|var| {
                        solution
                            .get(var.as_str())
                            .map(|term| term.to_string())
                            .unwrap_or_default()
                    })
                    .collect();
                rows.push(row);
            }

            match format {
                OutputFormat::Table => {
                    let mut builder = Builder::new();
                    builder.push_record(&vars);
                    for row in &rows {
                        builder.push_record(row);
                    }
                    let mut table = builder.build();
                    table.with(Style::rounded());
                    println!("{table}");
                }
                OutputFormat::Json => {
                    let json_rows: Vec<serde_json::Value> = rows
                        .iter()
                        .map(|row| {
                            let mut map = serde_json::Map::new();
                            for (var, val) in vars.iter().zip(row.iter()) {
                                map.insert(
                                    var.clone(),
                                    serde_json::Value::String(val.clone()),
                                );
                            }
                            serde_json::Value::Object(map)
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&json_rows)?);
                }
                OutputFormat::Csv => {
                    let mut wtr = csv::Writer::from_writer(std::io::stdout());
                    wtr.write_record(&vars)?;
                    for row in &rows {
                        wtr.write_record(row)?;
                    }
                    wtr.flush()?;
                }
            }
        }
    }

    Ok(())
}

/// Format a single scalar value.
pub fn format_scalar(label: &str, value: impl std::fmt::Display, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({ label: value.to_string() })
            );
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record([label]).ok();
            wtr.write_record([&value.to_string()]).ok();
            wtr.flush().ok();
        }
        OutputFormat::Table => {
            println!("{value}");
        }
    }
}

/// Print raw text (for N-Quads statements output etc).
pub fn format_raw(text: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({ "data": text })
            );
        }
        _ => {
            print!("{text}");
        }
    }
}
