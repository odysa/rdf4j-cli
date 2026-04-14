use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use crate::error::Rdf4jError;

const SPARQL_RESULTS_JSON: &str = "application/sparql-results+json";
const NQUADS: &str = "application/n-quads";
const TURTLE: &str = "text/turtle";
const SPARQL_UPDATE_CT: &str = "application/sparql-update";

#[derive(Default)]
pub struct StatementFilter {
    pub subj: Option<String>,
    pub pred: Option<String>,
    pub obj: Option<String>,
    pub context: Option<String>,
}

pub struct Rdf4jClient {
    http: Client,
    base_url: String,
}

impl Rdf4jClient {
    pub fn new(base_url: &str) -> Result<Self, Rdf4jError> {
        let http = Client::builder().build()?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn check(
        &self,
        resp: reqwest::blocking::Response,
    ) -> Result<reqwest::blocking::Response, Rdf4jError> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp)
        } else {
            let body = resp.text().unwrap_or_default();
            Err(Rdf4jError::ServerError {
                status: status.as_u16(),
                body,
            })
        }
    }

    pub fn protocol(&self) -> Result<String, Rdf4jError> {
        let resp = self.http.get(self.url("/protocol")).send()?;
        let resp = self.check(resp)?;
        Ok(resp.text()?)
    }

    pub fn health(&self) -> Result<bool, Rdf4jError> {
        match self.http.get(self.url("/protocol")).send() {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub fn list_repos(&self) -> Result<String, Rdf4jError> {
        let resp = self
            .http
            .get(self.url("/repositories"))
            .header(ACCEPT, SPARQL_RESULTS_JSON)
            .send()?;
        let resp = self.check(resp)?;
        Ok(resp.text()?)
    }

    pub fn create_repo(&self, id: &str, config_turtle: Vec<u8>) -> Result<(), Rdf4jError> {
        let resp = self
            .http
            .put(self.url(&format!("/repositories/{id}")))
            .header(CONTENT_TYPE, TURTLE)
            .body(config_turtle)
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn delete_repo(&self, id: &str) -> Result<(), Rdf4jError> {
        let resp = self
            .http
            .delete(self.url(&format!("/repositories/{id}")))
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn repo_size(&self, id: &str) -> Result<u64, Rdf4jError> {
        let resp = self
            .http
            .get(self.url(&format!("/repositories/{id}/size")))
            .send()?;
        let resp = self.check(resp)?;
        let text = resp.text()?;
        text.trim().parse().map_err(|_| Rdf4jError::ServerError {
            status: 0,
            body: format!("Invalid size response: {text}"),
        })
    }

    pub fn sparql_query(
        &self,
        repo_id: &str,
        query: &str,
        infer: bool,
    ) -> Result<String, Rdf4jError> {
        let resp = self
            .http
            .get(self.url(&format!("/repositories/{repo_id}")))
            .query(&[("query", query), ("infer", &infer.to_string())])
            .header(ACCEPT, SPARQL_RESULTS_JSON)
            .send()?;
        let resp = self.check(resp)?;
        Ok(resp.text()?)
    }

    pub fn sparql_update(&self, repo_id: &str, update: String) -> Result<(), Rdf4jError> {
        let resp = self
            .http
            .post(self.url(&format!("/repositories/{repo_id}/statements")))
            .header(CONTENT_TYPE, SPARQL_UPDATE_CT)
            .body(update)
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn get_statements(
        &self,
        repo_id: &str,
        filter: &StatementFilter,
        infer: bool,
    ) -> Result<String, Rdf4jError> {
        let infer_str = infer.to_string();
        let mut params = Self::filter_params(filter);
        params.push(("infer", infer_str.as_str()));

        let resp = self
            .http
            .get(self.url(&format!("/repositories/{repo_id}/statements")))
            .query(&params)
            .header(ACCEPT, NQUADS)
            .send()?;
        let resp = self.check(resp)?;
        Ok(resp.text()?)
    }

    pub fn add_statements(
        &self,
        repo_id: &str,
        body: impl Into<reqwest::blocking::Body>,
        content_type: &str,
        context: Option<&str>,
        base_uri: Option<&str>,
    ) -> Result<(), Rdf4jError> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(c) = context {
            params.push(("context", format!("<{c}>")));
        }
        if let Some(b) = base_uri {
            params.push(("baseURI", b.to_string()));
        }

        let resp = self
            .http
            .post(self.url(&format!("/repositories/{repo_id}/statements")))
            .header(CONTENT_TYPE, content_type)
            .query(&params)
            .body(body)
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn delete_statements(
        &self,
        repo_id: &str,
        filter: &StatementFilter,
    ) -> Result<(), Rdf4jError> {
        let params = Self::filter_params(filter);
        let resp = self
            .http
            .delete(self.url(&format!("/repositories/{repo_id}/statements")))
            .query(&params[..])
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn list_namespaces(&self, repo_id: &str) -> Result<String, Rdf4jError> {
        let resp = self
            .http
            .get(self.url(&format!("/repositories/{repo_id}/namespaces")))
            .header(ACCEPT, SPARQL_RESULTS_JSON)
            .send()?;
        let resp = self.check(resp)?;
        Ok(resp.text()?)
    }

    pub fn get_namespace(&self, repo_id: &str, prefix: &str) -> Result<String, Rdf4jError> {
        let resp = self
            .http
            .get(self.url(&format!("/repositories/{repo_id}/namespaces/{prefix}")))
            .send()?;
        let resp = self.check(resp)?;
        Ok(resp.text()?)
    }

    pub fn set_namespace(&self, repo_id: &str, prefix: &str, uri: &str) -> Result<(), Rdf4jError> {
        let resp = self
            .http
            .put(self.url(&format!("/repositories/{repo_id}/namespaces/{prefix}")))
            .body(uri.to_string())
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn delete_namespace(&self, repo_id: &str, prefix: &str) -> Result<(), Rdf4jError> {
        let resp = self
            .http
            .delete(self.url(&format!("/repositories/{repo_id}/namespaces/{prefix}")))
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    pub fn clear_namespaces(&self, repo_id: &str) -> Result<(), Rdf4jError> {
        let resp = self
            .http
            .delete(self.url(&format!("/repositories/{repo_id}/namespaces")))
            .send()?;
        self.check(resp)?;
        Ok(())
    }

    fn filter_params(filter: &StatementFilter) -> Vec<(&'static str, &str)> {
        let mut params = Vec::new();
        if let Some(s) = &filter.subj {
            params.push(("subj", s.as_str()));
        }
        if let Some(p) = &filter.pred {
            params.push(("pred", p.as_str()));
        }
        if let Some(o) = &filter.obj {
            params.push(("obj", o.as_str()));
        }
        if let Some(c) = &filter.context {
            params.push(("context", c.as_str()));
        }
        params
    }
}
