use rocket::request::Request;
use rocket::response;
use rocket::response::{Responder, Response};

use rocket::http::{ContentType, Status};

#[derive(Debug)]
pub struct ApiError {
    pub error: &'static str,
    pub status: Status,
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.error.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::Plain)
            .ok()
    }
}
