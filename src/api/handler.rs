// Placeholder for Request and Response
pub struct Request {
    pub method: String,
    pub path: String,
    pub body: String,
}

pub struct Response {
    pub status: u16,
    pub body: String,
    pub headers: std::collections::HashMap<String, String>,
}

pub fn handle_request(req: Request) -> Result<Response, String> {
    // Simple handler: echo body
    Ok(Response {
        status: 200,
        body: req.body,
        headers: [("Content-Type".to_string(), "application/json".to_string())].into(),
    })
}
