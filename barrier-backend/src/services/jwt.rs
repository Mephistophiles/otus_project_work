use crate::structs::Gate;
use jwt_simple::{
    algorithms::{HS256Key, MACLike},
    claims::Claims,
    common::VerificationOptions,
    prelude::Duration,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Jwt {
    key: String,
}

#[derive(Serialize, Deserialize)]
pub struct JWTToken {
    pub username: String,
    pub session_id: String,
    pub available_rooms: Vec<Gate>,
}

impl Jwt {
    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn issue_token(
        &self,
        username: String,
        available_rooms: Vec<Gate>,
        expired_in: Duration,
    ) -> (String, String, String) {
        let key = HS256Key::from_bytes(self.key.as_bytes());
        let refresh_token = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();

        let claims = JWTToken {
            username,
            session_id: session_id.clone(),
            available_rooms,
        };

        let claims = Claims::with_custom_claims(claims, expired_in);

        (
            key.authenticate(claims).expect("jwt token"),
            refresh_token,
            session_id,
        )
    }

    pub fn verify_token(&self, token: String) -> Option<JWTToken> {
        let key = HS256Key::from_bytes(self.key.as_bytes());

        let options = VerificationOptions {
            time_tolerance: Some(Duration::from_millis(0)),
            ..Default::default()
        };

        let claims = key.verify_token::<JWTToken>(&token, Some(options)).ok()?;

        Some(claims.custom)
    }
}
