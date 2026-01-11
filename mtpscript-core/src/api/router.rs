use regex::Regex;
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

#[derive(Debug)]
pub struct RouteMatch {
    pub api: ApiDeclaration,
    pub params: HashMap<String, String>,
}

pub struct Router {
    routes: Vec<CompiledRoute>,
}

struct CompiledRoute {
    method: HttpMethod,
    pattern: Regex,
    param_names: Vec<String>,
    api: ApiDeclaration,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route(&mut self, api: ApiDeclaration) {
        let (pattern, param_names) = Self::compile_path_pattern(&api.path);
        self.routes.push(CompiledRoute {
            method: api.method.clone(),
            pattern,
            param_names,
            api,
        });
    }

    pub fn from_apis(apis: &[ApiDeclaration]) -> Self {
        let mut router = Self::new();
        for api in apis {
            router.add_route(api.clone());
        }
        router
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<RouteMatch> {
        let method_enum = match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            _ => return None,
        };

        for route in &self.routes {
            if route.method != method_enum {
                continue;
            }

            if let Some(captures) = route.pattern.captures(path) {
                let mut params = HashMap::new();
                for (i, param_name) in route.param_names.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        params.insert(param_name.clone(), value.as_str().to_string());
                    }
                }

                return Some(RouteMatch {
                    api: route.api.clone(),
                    params,
                });
            }
        }

        None
    }

    fn compile_path_pattern(path: &str) -> (Regex, Vec<String>) {
        let mut param_names = Vec::new();
        let mut regex_pattern = String::from("^");

        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            regex_pattern.push('/');

            if segment.starts_with(':') {
                // Parameter segment
                let param_name = &segment[1..];
                param_names.push(param_name.to_string());
                regex_pattern.push_str(r"([^/]+)");
            } else {
                // Literal segment
                regex_pattern.push_str(&regex::escape(segment));
            }
        }

        regex_pattern.push('$');

        (Regex::new(&regex_pattern).unwrap(), param_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_routing() {
        let mut router = Router::new();
        let api = ApiDeclaration {
            method: HttpMethod::GET,
            path: "/users".to_string(),
            handler: "getUsers".to_string(),
        };
        router.add_route(api);

        let route_match = router.match_route("GET", "/users").unwrap();
        assert_eq!(route_match.api.method, HttpMethod::GET);
        assert_eq!(route_match.api.path, "/users");
        assert_eq!(route_match.api.handler, "getUsers");
        assert!(route_match.params.is_empty());
    }

    #[test]
    fn test_parameterized_routing() {
        let mut router = Router::new();
        let api = ApiDeclaration {
            method: HttpMethod::GET,
            path: "/users/:id".to_string(),
            handler: "getUser".to_string(),
        };
        router.add_route(api);

        let route_match = router.match_route("GET", "/users/123").unwrap();
        assert_eq!(route_match.api.handler, "getUser");
        assert_eq!(route_match.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_method_mismatch() {
        let mut router = Router::new();
        let api = ApiDeclaration {
            method: HttpMethod::POST,
            path: "/users".to_string(),
            handler: "createUser".to_string(),
        };
        router.add_route(api);

        assert!(router.match_route("GET", "/users").is_none());
    }

    #[test]
    fn test_multiple_parameters() {
        let mut router = Router::new();
        let api = ApiDeclaration {
            method: HttpMethod::GET,
            path: "/users/:userId/posts/:postId".to_string(),
            handler: "getUserPost".to_string(),
        };
        router.add_route(api);

        let route_match = router.match_route("GET", "/users/alice/posts/42").unwrap();
        assert_eq!(route_match.params.get("userId"), Some(&"alice".to_string()));
        assert_eq!(route_match.params.get("postId"), Some(&"42".to_string()));
    }

    #[test]
    fn test_from_apis() {
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

        let router = Router::from_apis(&apis);
        assert!(router.match_route("GET", "/users").is_some());
        assert!(router.match_route("POST", "/users").is_some());
        assert!(router.match_route("PUT", "/users").is_none());
    }
}
