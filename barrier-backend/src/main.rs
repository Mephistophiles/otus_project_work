use actix_web::{error, get, middleware::Logger, post, web, App, HttpRequest, HttpServer};
use chrono::Local;
use config::Config;
use jwt_simple::prelude::Duration;
use log::{debug, error, info};
use services::{
    auth::{Auth, LDAPAuth},
    db::{Db, MongoDb},
    jwt::{JWTToken, Jwt},
};
use std::sync::Arc;
use structs::{gates, login, logout, open, Errors, Gate};
use tokio::sync::Mutex;

use crate::services::db::EventType;

mod config;
mod middleware;
mod services;
mod structs;

pub(crate) fn configure_app(
    cfg: &mut web::ServiceConfig,
    auth: Arc<Mutex<Box<dyn Auth + Send>>>,
    db: Arc<Mutex<Box<dyn Db + Send>>>,
    config: Arc<Mutex<Config>>,
    jwt: Arc<Mutex<Jwt>>,
) {
    cfg.app_data(web::Data::new(auth))
        .app_data(web::Data::new(jwt))
        .app_data(web::Data::new(db))
        .app_data(web::Data::new(config))
        .service(
            web::scope("/auth/")
                .service(login_handler)
                .service(logout_handler)
                .service(refresh_handler),
        )
        .service(
            web::scope("/gates")
                .service(open_handler)
                .service(gates_handler),
        );
}

async fn issue_token(
    username: &str,
    rooms: Vec<Gate>,
    jwt: &Jwt,
    db: &dyn Db,
) -> (web::Json<login::Response>, String) {
    let (access_token, refresh_token, session_id) =
        jwt.issue_token(username.to_string(), rooms.clone(), Duration::from_mins(5));

    db.store_refresh(username, &refresh_token, &rooms).await;

    (
        web::Json(login::Response {
            access_token,
            refresh_token,
        }),
        session_id,
    )
}

#[post("/login")]
async fn login_handler(
    req: HttpRequest,
    data: web::Json<login::LoginRequest>,
    jwt: web::Data<Arc<Mutex<Jwt>>>,
    db: web::Data<Arc<Mutex<Box<dyn Db + Send>>>>,
    auth: web::Data<Arc<Mutex<Box<dyn Auth + Send>>>>,
) -> Result<web::Json<login::Response>, Errors> {
    info!("Authentication request for user {:?}", data.login);
    let auth = auth.lock().await;
    let jwt = jwt.lock().await;
    let db = db.lock().await;
    let rooms = auth.get_available_rooms(&data.login, &data.password);

    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("0.0.0.0")
        .to_string();

    match rooms {
        Some(rooms) => {
            let (token, session_id) = issue_token(&data.login, rooms, &jwt, db.as_ref()).await;
            info!(
                "Successful authentication for {:?} from {} at {}",
                data.login,
                ip,
                Local::now()
            );
            db.log_event(&ip, &data.login, &session_id, EventType::SuccessfulLogin)
                .await;
            Ok(token)
        }
        None => {
            error!(
                "Failed login for {:?} from {} at {}",
                data.login,
                ip,
                Local::now()
            );
            db.log_event(&ip, &data.login, "", EventType::FailedLogin)
                .await;
            Err(Errors::InvalidLogin)
        }
    }
}

#[post("/refresh")]
async fn refresh_handler(
    req: HttpRequest,
    data: web::Json<login::RefreshRequest>,
    jwt: web::Data<Arc<Mutex<Jwt>>>,
    db: web::Data<Arc<Mutex<Box<dyn Db + Send>>>>,
) -> actix_web::Result<web::Json<login::Response>> {
    let jwt = jwt.lock().await;
    let db = db.lock().await;

    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("0.0.0.0")
        .to_string();

    let user = match db.remove_by_refresh_token(&data.refresh_token).await {
        Some(user) => user,
        None => {
            error!("user with token {:?} not found", data.refresh_token);
            db.log_event(&ip, "", &data.refresh_token, EventType::FailedRefresh)
                .await;
            return Err(error::ErrorNotFound("Not Found"));
        }
    };

    let (token, session_id) = issue_token(&user.username, user.rooms, &jwt, db.as_ref()).await;

    info!(
        "Successful re-authentication for {:?} from {} at {}",
        user.username,
        ip,
        Local::now()
    );
    db.log_event(
        &ip,
        &user.username,
        &session_id,
        EventType::SuccessfulRefresh,
    )
    .await;

    Ok(token)
}

#[post("/logout")]
async fn logout_handler(
    jwt: JWTToken,
    db: web::Data<Arc<Mutex<Box<dyn Db + Send>>>>,
) -> Result<web::Json<logout::Response>, Errors> {
    let db = db.lock().await;

    db.remove_by_username(&jwt.username).await;
    Ok(web::Json(logout::Response { success: true }))
}

#[post("/open/{gate}")]
async fn open_handler(
    req: HttpRequest,
    gate: web::Path<(String,)>,
    config: web::Data<Arc<Mutex<Config>>>,
    db: web::Data<Arc<Mutex<Box<dyn Db + Send>>>>,
    jwt: JWTToken,
) -> Result<web::Json<open::Response>, Errors> {
    let db = db.lock().await;
    let config = config.lock().await;

    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("0.0.0.0")
        .to_string();

    let current_gate = jwt.available_rooms.iter().find(|g| gate.0 == g.name);

    if let Some(current_gate) = current_gate {
        let success = if config.dry_run {
            debug!(
                "emulate open gate {} with {} retries",
                current_gate.id, current_gate.retries
            );
            true
        } else {
            services::gate::open(&config.gate_server, current_gate.id, current_gate.retries)
                .await
                .is_ok()
        };
        info!(
            "Successful access to gate {} for {:?} from {} at {}",
            gate.0,
            jwt.username,
            ip,
            Local::now()
        );
        db.log_event(
            &ip,
            &jwt.username,
            &jwt.session_id,
            EventType::SuccessfulGateAccess {
                gate: gate.0.clone(),
            },
        )
        .await;
        Ok(web::Json(open::Response { success }))
    } else {
        error!(
            "Unauthorized access to gate {} for {:?} from {} at {}",
            gate.0,
            jwt.username,
            ip,
            Local::now()
        );
        db.log_event(
            &ip,
            &jwt.username,
            &jwt.session_id,
            EventType::UnauthorizedGateAccess {
                gate: gate.0.clone(),
            },
        )
        .await;
        Err(Errors::Unauthorized)
    }
}

#[get("/list")]
async fn gates_handler(jwt: JWTToken) -> Result<web::Json<gates::Response>, Errors> {
    Ok(web::Json(gates::Response {
        gates: jwt.available_rooms,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::new();
    flexi_logger::Logger::try_with_env_or_str(&config.log_level)
        .expect("logger")
        .start()
        .expect("logger");

    let jwt = Arc::new(Mutex::new(Jwt::new(config.jwt_key.clone())));
    let mongo_db: Box<dyn Db + Send> = Box::new(MongoDb::new(&config.mongo_uri).await);
    let db = Arc::new(Mutex::new(mongo_db));
    let auth = LDAPAuth::new(
        config.ldap.server.clone(),
        config.ldap.base.clone(),
        config.ldap.bind.clone(),
        config.ldap.filter.clone(),
        config.get_mappings(),
    );
    let auth: Box<dyn Auth + Send> = Box::new(auth);
    let auth = Arc::new(Mutex::new(auth));
    let listen_addr = config.listen_addr.clone();
    let config = Arc::new(Mutex::new(config));

    HttpServer::new(move || {
        let jwt = jwt.clone();
        let db = db.clone();
        let auth = auth.clone();
        let config = config.clone();
        App::new()
            .wrap(Logger::default())
            .configure(move |cfg| configure_app(cfg, auth, db, config, jwt))
    })
    .bind(&listen_addr)?
    .run()
    .await
}

#[cfg(test)]
mod tests;
