use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Debug, Clone)]
pub struct ApiDeclaration {
    pub method: HttpMethod,
    pub path: String,
    pub handler: String, // Function name
}

pub struct Router {
    routes: HashMap<String, ApiDeclaration>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, api: ApiDeclaration) {
        let key = format!("{:?}:{}", api.method, api.path);
        self.routes.insert(key, api);
    }

    pub fn from_apis(apis: &[ApiDeclaration]) -> Self {
        let mut router = Self::new();
        for api in apis {
            router.add_route(api.clone());
        }
        router
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<&ApiDeclaration> {
        let method_enum = match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            _ => return None,
        };

        let key = format!("{:?}:{}", method_enum, path);
        self.routes.get(&key)
    }
}
