use actix_web::{
    error,
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Error)]
pub enum Errors {
    #[display(fmt = "Invalid login or password")]
    InvalidLogin,
    #[display(fmt = "Unauthorized access")]
    Unauthorized,
}

#[derive(Serialize)]
pub struct CommonError {
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gate {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub retries: i32,
}

impl PartialEq for Gate {
    fn eq(&self, rhs: &Gate) -> bool {
        self.id == rhs.id
    }
}

impl Eq for Gate {}

impl error::ResponseError for Errors {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .json(CommonError {
                error: self.to_string(),
            })
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Errors::InvalidLogin => StatusCode::FORBIDDEN,
            Errors::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

pub mod login {
    use super::*;

    #[derive(PartialEq, Debug, Eq, Serialize, Deserialize)]
    pub struct LoginRequest {
        pub login: String,
        pub password: String,
    }

    #[derive(PartialEq, Debug, Eq, Serialize, Deserialize)]
    pub struct RefreshRequest {
        pub refresh_token: String,
    }

    #[derive(PartialEq, Debug, Eq, Serialize, Deserialize)]
    pub struct Response {
        pub access_token: String,
        pub refresh_token: String,
    }
}

pub mod logout {
    use super::*;

    #[derive(PartialEq, Debug, Eq, Serialize, Deserialize)]
    pub struct Response {
        pub success: bool,
    }
}

pub mod open {
    use super::*;

    #[derive(PartialEq, Debug, Eq, Serialize, Deserialize)]
    pub struct Response {
        pub success: bool,
    }
}

pub mod gates {
    use super::*;

    #[derive(PartialEq, Debug, Eq, Serialize, Deserialize)]
    pub struct Response {
        pub gates: Vec<Gate>,
    }
}
