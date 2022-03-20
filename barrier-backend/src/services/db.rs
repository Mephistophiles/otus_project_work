use mongodb::{bson::doc, Client, Database};
use serde::{Deserialize, Serialize};

use crate::structs::Gate;

pub struct MongoDb {
    db: Database,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct RefreshTokenItem {
    pub username: String,
    pub refresh_token: String,
    pub rooms: Vec<Gate>,
}

#[derive(Serialize, Deserialize)]
struct EventLog<'a> {
    ip: &'a str,
    username: &'a str,
    event_type: &'static str,
    date: mongodb::bson::DateTime,
    session_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    gate: Option<String>,
}

pub enum EventType {
    SuccessfulLogin,
    FailedLogin,
    SuccessfulRefresh,
    FailedRefresh,
    SuccessfulGateAccess { gate: String },
    UnauthorizedGateAccess { gate: String },
}

fn event_to_log<'a>(
    ip: &'a str,
    username: &'a str,
    session_id: &'a str,
    event: EventType,
) -> EventLog<'a> {
    match event {
        EventType::SuccessfulLogin => EventLog {
            ip,
            username,
            event_type: "Successful login",
            date: mongodb::bson::DateTime::now(),
            session_id,
            gate: None,
        },
        EventType::FailedLogin => EventLog {
            ip,
            username,
            event_type: "Failed login",
            date: mongodb::bson::DateTime::now(),
            session_id,
            gate: None,
        },
        EventType::SuccessfulRefresh => EventLog {
            ip,
            username,
            event_type: "Successful refresh token",
            date: mongodb::bson::DateTime::now(),
            session_id,
            gate: None,
        },
        EventType::FailedRefresh => EventLog {
            ip,
            username,
            event_type: "Failed refresh token",
            date: mongodb::bson::DateTime::now(),
            session_id,
            gate: None,
        },
        EventType::SuccessfulGateAccess { gate } => EventLog {
            ip,
            username,
            event_type: "Successful access to gate",
            date: mongodb::bson::DateTime::now(),
            session_id,
            gate: Some(gate),
        },
        EventType::UnauthorizedGateAccess { gate } => EventLog {
            ip,
            username,
            event_type: "Unauthorized gate access",
            date: mongodb::bson::DateTime::now(),
            session_id,
            gate: Some(gate),
        },
    }
}
#[async_trait::async_trait]
pub trait Db {
    async fn log_event(&self, ip: &str, username: &str, session_id: &str, event: EventType);
    async fn store_refresh(&self, username: &str, refresh_token: &str, rooms: &[Gate]);
    async fn remove_by_refresh_token(&self, refresh_token: &str) -> Option<RefreshTokenItem>;
    async fn remove_by_username(&self, username: &str);
}

impl MongoDb {
    pub async fn new(uri: &str) -> Self {
        let client = Client::with_uri_str(uri).await.expect("DB client");
        let db = client.database("barrier");

        db.run_command(
            doc! {
                "createIndexes": "refresh_tokens",
                "indexes": [
                    {
                        "key": { "refresh_token": 1 },
                        "name": "refresh_token_index",
                        "expireAfterSeconds": 1209600, // 2 weeks
                        "unique": true
                    },
                ]
            },
            None,
        )
        .await
        .unwrap();

        Self { db }
    }
}

#[async_trait::async_trait]
impl Db for MongoDb {
    async fn log_event(&self, ip: &str, username: &str, session_id: &str, event: EventType) {
        let audit_log = self.db.collection::<EventLog>("audit");

        audit_log
            .insert_one(event_to_log(ip, username, session_id, event), None)
            .await
            .expect("insert log");
    }
    async fn store_refresh(&self, username: &str, refresh_token: &str, rooms: &[Gate]) {
        let refresh_tokens = self.db.collection::<RefreshTokenItem>("refresh_tokens");

        refresh_tokens
            .insert_one(
                RefreshTokenItem {
                    username: username.to_string(),
                    refresh_token: refresh_token.to_string(),
                    rooms: rooms.to_vec(),
                },
                None,
            )
            .await
            .expect("Insert");
    }

    async fn remove_by_refresh_token(&self, refresh_token: &str) -> Option<RefreshTokenItem> {
        let refresh_tokens = self.db.collection::<RefreshTokenItem>("refresh_tokens");

        refresh_tokens
            .find_one_and_delete(
                doc! {
                            "refresh_token": refresh_token

                },
                None,
            )
            .await
            .expect("Normal db connection")
    }

    async fn remove_by_username(&self, username: &str) {
        let refresh_tokens = self.db.collection::<RefreshTokenItem>("refresh_tokens");

        refresh_tokens
            .delete_many(
                doc! {
                    "username": username
                },
                None,
            )
            .await
            .expect("Normal delete one");
    }
}

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use tokio::sync::Mutex;

#[cfg(test)]
pub struct Cache {
    cache: Mutex<HashMap<String, RefreshTokenItem>>,
}

#[cfg(test)]
impl Cache {
    pub async fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl Db for Cache {
    async fn log_event(&self, _: &str, _: &str, _: &str, _: EventType) {}

    async fn store_refresh(&self, username: &str, refresh_token: &str, rooms: &[Gate]) {
        self.cache.lock().await.insert(
            refresh_token.to_string(),
            RefreshTokenItem {
                username: username.to_string(),
                refresh_token: refresh_token.to_string(),
                rooms: rooms.to_vec(),
            },
        );
    }

    async fn remove_by_refresh_token(&self, refresh_token: &str) -> Option<RefreshTokenItem> {
        self.cache.lock().await.remove(refresh_token)
    }

    async fn remove_by_username(&self, username: &str) {
        self.cache
            .lock()
            .await
            .retain(|_, u| u.username != username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn cache_test() {
        let cache = Cache::new().await;

        assert_eq!(cache.cache.lock().await.len(), 0);

        assert_eq!(
            cache.remove_by_refresh_token("INVALID_REFRESH_TOKEN").await,
            None
        );

        cache
            .store_refresh(
                "admin",
                "VALID_REFRESH_TOKEN",
                &[Gate {
                    id: 1,
                    retries: 1,
                    name: "room".to_string(),
                    description: "".to_string(),
                }],
            )
            .await;

        assert_eq!(cache.cache.lock().await.len(), 1);

        assert_eq!(
            cache.remove_by_refresh_token("VALID_REFRESH_TOKEN").await,
            Some(RefreshTokenItem {
                username: "admin".to_string(),
                refresh_token: "VALID_REFRESH_TOKEN".to_string(),
                rooms: vec![Gate {
                    id: 1,
                    retries: 1,
                    name: "room".to_string(),
                    description: "".to_string(),
                }],
            })
        );

        assert_eq!(cache.cache.lock().await.len(), 0);

        cache
            .store_refresh(
                "admin",
                "VALID_REFRESH_TOKEN1",
                &[Gate {
                    id: 1,
                    retries: 1,
                    name: "room".to_string(),
                    description: "".to_string(),
                }],
            )
            .await;
        cache
            .store_refresh(
                "admin",
                "VALID_REFRESH_TOKEN2",
                &[Gate {
                    id: 1,
                    retries: 1,
                    name: "room".to_string(),
                    description: "".to_string(),
                }],
            )
            .await;
        assert_eq!(cache.cache.lock().await.len(), 2);
        cache.remove_by_username("admin").await;
        assert_eq!(cache.cache.lock().await.len(), 0);
    }
}
