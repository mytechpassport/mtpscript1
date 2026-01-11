use std::collections::HashMap;

use crate::api::router::{ApiDeclaration, HttpMethod};

#[derive(Debug, serde::Serialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: HashMap<String, HashMap<String, OpenApiOperation>>,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiOperation {
    pub responses: HashMap<String, OpenApiResponse>,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiResponse {
    pub description: String,
}

pub fn generate_openapi(apis: &[ApiDeclaration]) -> OpenApiSpec {
    let mut paths = HashMap::new();

    // Sort APIs by path for deterministic output
    let mut sorted_apis = apis.to_vec();
    sorted_apis.sort_by(|a, b| a.path.cmp(&b.path));

    for api in sorted_apis {
        let method = match api.method {
            HttpMethod::GET => "get",
            HttpMethod::POST => "post",
            HttpMethod::PUT => "put",
            HttpMethod::DELETE => "delete",
            HttpMethod::PATCH => "patch",
        };

        let operation = OpenApiOperation {
            responses: HashMap::from([(
                "200".to_string(),
                OpenApiResponse {
                    description: "Success".to_string(),
                },
            )]),
        };

        paths
            .entry(api.path)
            .or_insert_with(HashMap::new)
            .insert(method.to_string(), operation);
    }

    OpenApiSpec {
        openapi: "3.0.0".to_string(),
        info: OpenApiInfo {
            title: "MTPScript API".to_string(),
            version: "1.0.0".to_string(),
        },
        paths,
    }
}
