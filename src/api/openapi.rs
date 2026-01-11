use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct OpenApi {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Components,
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    // etc.
}

#[derive(Serialize, Deserialize)]
pub struct Operation {
    pub responses: HashMap<String, Response>,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: HashMap<String, MediaType>,
}

#[derive(Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
}

#[derive(Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    // etc.
}

#[derive(Serialize, Deserialize)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
}

pub fn generate_openapi(routes: Vec<Route>) -> OpenApi {
    let mut paths = HashMap::new();

    for route in routes {
        let path_item = paths.entry(route.path.clone()).or_insert(PathItem {
            get: None,
            post: None,
        });

        let operation = Operation {
            responses: {
                let mut responses = HashMap::new();
                responses.insert(
                    "200".to_string(),
                    Response {
                        description: "Successful response".to_string(),
                        content: {
                            let mut content = HashMap::new();
                            content.insert(
                                "application/json".to_string(),
                                MediaType {
                                    schema: Schema {
                                        type_: Some("object".to_string()),
                                        properties: None,
                                    },
                                },
                            );
                            content
                        },
                    },
                );
                responses
            },
        };

        match route.method.to_uppercase().as_str() {
            "GET" => path_item.get = Some(operation),
            "POST" => path_item.post = Some(operation),
            // Add other methods as needed
            _ => {}
        }
    }

    OpenApi {
        openapi: "3.0.0".to_string(),
        info: Info {
            title: "MTPScript API".to_string(),
            version: "1.0.0".to_string(),
        },
        paths,
        components: Components {
            schemas: HashMap::new(),
        },
    }
}

// Placeholder for Route, assume imported or defined
pub struct Route {
    pub method: String,
    pub path: String,
}
