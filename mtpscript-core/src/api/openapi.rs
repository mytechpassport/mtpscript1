use crate::api::router::{ApiDeclaration, HttpMethod};
use crate::types::{AdtVariant, Type};
use std::collections::BTreeMap;

#[derive(Debug, serde::Serialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: BTreeMap<String, BTreeMap<String, OpenApiOperation>>,
    #[serde(skip_serializing_if = "OpenApiComponents::is_empty")]
    pub components: OpenApiComponents,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiOperation {
    pub responses: BTreeMap<String, OpenApiResponse>,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiResponse {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<BTreeMap<String, OpenApiMediaType>>,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiMediaType {
    pub schema: OpenApiSchema,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenApiComponents {
    pub schemas: BTreeMap<String, OpenApiSchema>,
}

impl OpenApiComponents {
    fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum OpenApiSchema {
    Ref {
        r#ref: String,
    },
    Object {
        #[serde(rename = "type")]
        schema_type: String,
        properties: BTreeMap<String, OpenApiSchema>,
        required: Vec<String>,
    },
    OneOf {
        one_of: Vec<OpenApiSchema>,
    },
    Array {
        #[serde(rename = "type")]
        schema_type: String,
        items: Box<OpenApiSchema>,
    },
    Primitive {
        #[serde(rename = "type")]
        schema_type: String,
    },
}

pub fn generate_openapi(apis: &[ApiDeclaration], types: &[Type]) -> OpenApiSpec {
    let mut paths = BTreeMap::new();
    let mut schemas = BTreeMap::new();

    // Add Json schema (required for responses)
    schemas.insert(
        "Json".to_string(),
        OpenApiSchema::Ref {
            r#ref: "#/components/schemas/JsonValue".to_string(),
        },
    );

    // Json ADT schema
    schemas.insert(
        "JsonValue".to_string(),
        OpenApiSchema::OneOf {
            one_of: vec![
                OpenApiSchema::Primitive {
                    schema_type: "null".to_string(),
                },
                OpenApiSchema::Primitive {
                    schema_type: "boolean".to_string(),
                },
                OpenApiSchema::Primitive {
                    schema_type: "number".to_string(),
                },
                OpenApiSchema::Primitive {
                    schema_type: "string".to_string(),
                },
                OpenApiSchema::Object {
                    schema_type: "object".to_string(),
                    properties: BTreeMap::new(),
                    required: vec![],
                },
                OpenApiSchema::Array {
                    schema_type: "array".to_string(),
                    items: Box::new(OpenApiSchema::Ref {
                        r#ref: "#/components/schemas/JsonValue".to_string(),
                    }),
                },
            ],
        },
    );

    // Generate schemas for all types
    for typ in types {
        add_type_to_schemas(typ, &mut schemas);
    }

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
            responses: BTreeMap::from([(
                "200".to_string(),
                OpenApiResponse {
                    description: "Success".to_string(),
                    content: Some(BTreeMap::from([(
                        "application/json".to_string(),
                        OpenApiMediaType {
                            schema: OpenApiSchema::Ref {
                                r#ref: "#/components/schemas/Json".to_string(),
                            },
                        },
                    )])),
                },
            )]),
        };

        paths
            .entry(api.path)
            .or_insert_with(BTreeMap::new)
            .insert(method.to_string(), operation);
    }

    OpenApiSpec {
        openapi: "3.0.0".to_string(),
        info: OpenApiInfo {
            title: "MTPScript API".to_string(),
            version: "1.0.0".to_string(),
        },
        paths,
        components: OpenApiComponents { schemas },
    }
}

fn add_type_to_schemas(typ: &Type, schemas: &mut BTreeMap<String, OpenApiSchema>) {
    match typ {
        Type::Record(record) => {
            let ref_name = format!("Record{}", sha256_ref(&record.name));
            if schemas.contains_key(&ref_name) {
                return;
            }

            let mut properties = BTreeMap::new();
            let mut required = Vec::new();

            for (field_name, field_type) in &record.fields {
                let field_schema = type_to_schema(field_type, schemas);
                properties.insert(field_name.clone(), field_schema);
                required.push(field_name.clone());
            }

            schemas.insert(
                ref_name.clone(),
                OpenApiSchema::Object {
                    schema_type: "object".to_string(),
                    properties,
                    required,
                },
            );
        }
        Type::Adt(adt) => {
            let ref_name = format!("Adt{}", sha256_ref(&adt.content_hash()));
            if schemas.contains_key(&ref_name) {
                return;
            }

            let mut one_of = Vec::new();

            for variant in &adt.variants {
                match variant {
                    AdtVariant::Unit(_name) => {
                        // Unit variant as object with tag
                        let mut properties = BTreeMap::new();
                        properties.insert(
                            "tag".to_string(),
                            OpenApiSchema::Primitive {
                                schema_type: "string".to_string(),
                            },
                        );
                        one_of.push(OpenApiSchema::Object {
                            schema_type: "object".to_string(),
                            properties,
                            required: vec!["tag".to_string()],
                        });
                    }
                    AdtVariant::Tuple(_name, types) => {
                        // Tuple variant as object with tag and value array
                        let mut properties = BTreeMap::new();
                        properties.insert(
                            "tag".to_string(),
                            OpenApiSchema::Primitive {
                                schema_type: "string".to_string(),
                            },
                        );

                        let items = if types.len() == 1 {
                            type_to_schema(&types[0], schemas)
                        } else {
                            // For multiple types, use array
                            OpenApiSchema::Array {
                                schema_type: "array".to_string(),
                                items: Box::new(OpenApiSchema::Primitive {
                                    schema_type: "string".to_string(), // Simplified
                                }),
                            }
                        };

                        properties.insert("value".to_string(), items);

                        one_of.push(OpenApiSchema::Object {
                            schema_type: "object".to_string(),
                            properties,
                            required: vec!["tag".to_string(), "value".to_string()],
                        });
                    }
                }
            }

            schemas.insert(ref_name.clone(), OpenApiSchema::OneOf { one_of });
        }
        _ => {} // Other types handled inline
    }
}

fn type_to_schema(typ: &Type, schemas: &mut BTreeMap<String, OpenApiSchema>) -> OpenApiSchema {
    match typ {
        Type::Number => OpenApiSchema::Primitive {
            schema_type: "number".to_string(),
        },
        Type::Boolean => OpenApiSchema::Primitive {
            schema_type: "boolean".to_string(),
        },
        Type::String => OpenApiSchema::Primitive {
            schema_type: "string".to_string(),
        },
        Type::Decimal => OpenApiSchema::Primitive {
            schema_type: "string".to_string(), // Decimal as string in JSON
        },
        Type::TypeVar(_) => OpenApiSchema::Primitive {
            schema_type: "string".to_string(), // Type variable fallback
        },
        Type::Json => OpenApiSchema::Ref {
            r#ref: "#/components/schemas/Json".to_string(),
        },
        Type::Record(record) => {
            let ref_name = format!("Record{}", sha256_ref(&record.name));
            add_type_to_schemas(typ, schemas);
            OpenApiSchema::Ref {
                r#ref: format!("#/components/schemas/{}", ref_name),
            }
        }
        Type::Adt(adt) => {
            let ref_name = format!("Adt{}", sha256_ref(&adt.content_hash()));
            add_type_to_schemas(typ, schemas);
            OpenApiSchema::Ref {
                r#ref: format!("#/components/schemas/{}", ref_name),
            }
        }
        Type::Var(_) => OpenApiSchema::Primitive {
            schema_type: "string".to_string(), // Fallback
        },
        Type::Function(_, _) => OpenApiSchema::Primitive {
            schema_type: "string".to_string(), // Functions not represented in OpenAPI
        },
    }
}

fn sha256_ref(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let apis = vec![
            ApiDeclaration {
                method: HttpMethod::GET,
                path: "/users".to_string(),
                handler: "getUsers".to_string(),
            },
            ApiDeclaration {
                method: HttpMethod::POST,
                path: "/users".to_string(),
                handler: "createUser".to_string(),
            },
        ];

        let types = vec![Type::Json]; // Include Json type

        let spec = generate_openapi(&apis, &types);

        assert_eq!(spec.openapi, "3.0.0");
        assert_eq!(spec.info.title, "MTPScript API");
        assert!(spec.paths.contains_key("/users"));
        assert!(spec.components.schemas.contains_key("JsonValue"));
    }

    #[test]
    fn test_deterministic_output() {
        let apis = vec![
            ApiDeclaration {
                method: HttpMethod::GET,
                path: "/b".to_string(),
                handler: "handler2".to_string(),
            },
            ApiDeclaration {
                method: HttpMethod::GET,
                path: "/a".to_string(),
                handler: "handler1".to_string(),
            },
        ];

        let types = vec![];

        let spec1 = generate_openapi(&apis, &types);
        let spec2 = generate_openapi(&apis, &types);

        // Should be identical
        assert_eq!(
            serde_json::to_string(&spec1).unwrap(),
            serde_json::to_string(&spec2).unwrap()
        );
    }

    #[test]
    fn test_sha256_ref() {
        let ref1 = sha256_ref("test");
        let ref2 = sha256_ref("test");
        assert_eq!(ref1, ref2);
        assert_eq!(ref1.len(), 64); // SHA-256 hex length
    }
}
