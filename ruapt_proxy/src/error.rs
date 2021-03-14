use actix_web::{HttpResponse, ResponseError};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ProxyError {}

impl Display for ProxyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ProxyError {}

impl ResponseError for ProxyError {
    /// Transform error messages to Http Response.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body("TODO")
    }
}
