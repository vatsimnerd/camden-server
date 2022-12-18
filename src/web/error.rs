use rocket::{
  catch,
  http::Status,
  response::{status::Custom, Responder},
  serde::json::json,
};

use crate::lee::parser::error::{CompileError, ParseError};

#[derive(Debug)]
pub struct APIError {
  pub code: u16,
  pub message: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for APIError {
  fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
    let resp = Custom(
      Status { code: self.code },
      json!({
        "error": self.message
      }),
    );
    resp.respond_to(request)
  }
}

impl From<ParseError> for APIError {
  fn from(err: ParseError) -> Self {
    APIError {
      code: 400,
      message: format!("error parsing query: {}", err),
    }
  }
}

impl From<CompileError> for APIError {
  fn from(err: CompileError) -> Self {
    APIError {
      code: 400,
      message: format!("error compiling query: {}", err),
    }
  }
}

impl From<mongodb::error::Error> for APIError {
  fn from(err: mongodb::error::Error) -> Self {
    APIError {
      code: 500,
      message: format!("{}", err),
    }
  }
}

pub fn api_error(code: u16, message: &str) -> APIError {
  APIError {
    code,
    message: message.into(),
  }
}

pub fn not_found(message: &str) -> APIError {
  api_error(404, message)
}

pub fn bad_request(message: &str) -> APIError {
  api_error(403, message)
}

#[catch(404)]
pub fn catch404() -> APIError {
  not_found("not found")
}

#[catch(500)]
pub fn catch500() -> APIError {
  api_error(500, "internal server error")
}
