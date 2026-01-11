use std::collections::HashMap;

pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: Box<dyn Fn() -> ()>, // placeholder
}

pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<(&Route, std::collections::HashMap<String, String>)> {
        for route in &self.routes {
            if route.method == method {
                if let Some(params) = Self::match_path(&route.path, path) {
                    return Some((route, params));
                }
            }
        }
        None
    }

    fn match_path(route_path: &str, request_path: &str) -> Option<std::collections::HashMap<String, String>> {
        let route_segments: Vec<&str> = route_path.split('/').filter(|s| !s.is_empty()).collect();
        let request_segments: Vec<&str> = request_path.split('/').filter(|s| !s.is_empty()).collect();

        if route_segments.len() != request_segments.len() {
            return None;
        }

        let mut params = std::collections::HashMap::new();

        for (route_seg, req_seg) in route_segments.iter().zip(request_segments.iter()) {
            if route_seg.starts_with(':') {
                let param_name = &route_seg[1..];
                params.insert(param_name.to_string(), req_seg.to_string());
            } else if route_seg != req_seg {
                return None;
            }
        }

        Some(params)
    }
}
