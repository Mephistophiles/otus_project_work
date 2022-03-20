use super::*;
use crate::services::db::Cache;
use actix_web::{http::StatusCode, test};
use services::auth::FakeAuth;

macro_rules! init_test_env {
    () => {{
        flexi_logger::Logger::try_with_env_or_str("crit")
            .unwrap()
            .start()
            .ok();

        let jwt = Jwt::new(JWT_SIGN_KEY.to_string());
        let cache = Cache::new().await;
        let mut auth = FakeAuth::new();
        let mut config = Config::default();
        config.dry_run = true;

        auth.add_user(
            LOGIN_1,
            PASSWORD_1,
            &[
                Gate {
                    id: 1,
                    retries: 1,
                    name: "".to_string(),
                    description: "".to_string(),
                },
                Gate {
                    id: 2,
                    retries: 1,
                    name: "".to_string(),
                    description: "".to_string(),
                },
            ],
        );
        cache
            .store_refresh(
                LOGIN_1,
                REFRESH_TOKEN_1,
                &[
                    Gate {
                        id: 1,
                        retries: 1,
                        name: "".to_string(),
                        description: "".to_string(),
                    },
                    Gate {
                        id: 2,
                        retries: 1,
                        name: "".to_string(),
                        description: "".to_string(),
                    },
                ],
            )
            .await;

        let auth: Box<dyn Auth + Send> = Box::new(auth);
        let auth = Arc::new(Mutex::new(auth));
        let cache: Box<dyn Db + Send> = Box::new(cache);
        let cache = Arc::new(Mutex::new(cache));
        let config = Arc::new(Mutex::new(config));
        let jwt = Arc::new(Mutex::new(jwt));

        let app = App::new().configure(move |cfg| configure_app(cfg, auth, cache, config, jwt));

        test::init_service(app).await
    }};
}

macro_rules! login {
    ($app:ident, $login:ident, $password:ident) => {{
        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&login::LoginRequest {
                login: $login.to_string(),
                password: $password.to_string(),
            })
            .to_request();

        let resp = test::call_service(&$app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: login::Response = test::read_body_json(resp).await;

        assert!(!body.access_token.is_empty());
        assert!(!body.refresh_token.is_empty());

        body
    }};
}

const LOGIN_1: &str = "login1";
const PASSWORD_1: &str = "password1";

const REFRESH_TOKEN_1: &str = "REFRESH_TOKEN_1";

const JWT_SIGN_KEY: &str = "jwt";

// auth

#[actix_rt::test]
async fn got_200_on_success_login() {
    let app = init_test_env!();

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login::LoginRequest {
            login: LOGIN_1.to_string(),
            password: PASSWORD_1.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: login::Response = test::read_body_json(resp).await;

    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());

    // can get data
    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {}", body.access_token)))
        .uri("/gates/list")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // can refresh

    let req = test::TestRequest::post()
        .uri("/auth/refresh")
        .set_json(&login::RefreshRequest {
            refresh_token: body.refresh_token,
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: login::Response = test::read_body_json(resp).await;

    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());
}

#[actix_rt::test]
async fn got_403_on_invalid_login() {
    let app = init_test_env!();

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login::LoginRequest {
            login: "NOT_VALID_LOGIN".to_string(),
            password: "NOT_VALID_PASSWORD".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn got_200_on_success_refresh() {
    let app = init_test_env!();

    let req = test::TestRequest::post()
        .uri("/auth/refresh")
        .set_json(&login::RefreshRequest {
            refresh_token: REFRESH_TOKEN_1.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: login::Response = test::read_body_json(resp).await;

    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());
}

#[actix_rt::test]
async fn multiple_refresh_tokens_are_valid() {
    let app = init_test_env!();

    let body1 = login!(app, LOGIN_1, PASSWORD_1);
    let body2 = login!(app, LOGIN_1, PASSWORD_1);

    let req1 = test::TestRequest::post()
        .set_json(&login::RefreshRequest {
            refresh_token: body1.refresh_token,
        })
        .uri("/auth/refresh")
        .to_request();

    let req2 = test::TestRequest::post()
        .set_json(&login::RefreshRequest {
            refresh_token: body2.refresh_token,
        })
        .uri("/auth/refresh")
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let resp2 = test::call_service(&app, req2).await;

    assert_eq!(resp1.status(), StatusCode::OK);
    assert_eq!(resp2.status(), StatusCode::OK);

    let body1: login::Response = test::read_body_json(resp1).await;
    let body2: login::Response = test::read_body_json(resp2).await;

    assert!(!body1.access_token.is_empty());
    assert!(!body1.refresh_token.is_empty());
    assert!(!body2.access_token.is_empty());
    assert!(!body2.refresh_token.is_empty());
}

#[actix_rt::test]
async fn got_200_on_success_logout() {
    let app = init_test_env!();

    let body = login!(app, LOGIN_1, PASSWORD_1);

    let req = test::TestRequest::post()
        .insert_header(("Authorization", format!("Bearer {}", body.access_token)))
        .uri("/auth/logout")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn got_401_on_failed_logout() {
    let app = init_test_env!();

    let req = test::TestRequest::post()
        .insert_header(("Authorization", format!("Bearer {}", "NOT_VALID_TOKEN")))
        .uri("/auth/logout")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn use_refresh_token_only_once() {
    let app = init_test_env!();

    let req = test::TestRequest::post()
        .uri("/auth/refresh")
        .set_json(&login::RefreshRequest {
            refresh_token: REFRESH_TOKEN_1.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::post()
        .uri("/auth/refresh")
        .set_json(&login::RefreshRequest {
            refresh_token: REFRESH_TOKEN_1.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn got_404_on_invalid_refresh() {
    let app = init_test_env!();
    let req = test::TestRequest::post()
        .uri("/auth/refresh")
        .set_json(&login::RefreshRequest {
            refresh_token: "INVALID_REFRESH_TOKEN".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn got_401_on_expired_token() {
    let app = init_test_env!();

    let token = Jwt::new(JWT_SIGN_KEY.to_string()).issue_token(
        "admin".to_string(),
        [
            Gate {
                id: 1,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 2,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
        ]
        .to_vec(),
        Duration::from_secs(0),
    );

    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {}", token.0)))
        .uri("/gates/list")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn got_403_on_expired_token() {
    let app = init_test_env!();

    let token = Jwt::new(JWT_SIGN_KEY.to_string()).issue_token(
        "admin".to_string(),
        [
            Gate {
                id: 1,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 2,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
        ]
        .to_vec(),
        Duration::from_secs(0),
    );

    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {}", token.0)))
        .uri("/gates/list")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// end auth

#[actix_rt::test]
async fn open_the_gate() {
    let app = init_test_env!();

    let token = Jwt::new(JWT_SIGN_KEY.to_string()).issue_token(
        "admin".to_string(),
        [
            Gate {
                id: 1,
                retries: 1,
                name: "bathroom".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 2,
                retries: 1,
                name: "kitchen".to_string(),
                description: "".to_string(),
            },
        ]
        .to_vec(),
        Duration::from_secs(60),
    );

    // open the available room
    let req = test::TestRequest::post()
        .insert_header(("Authorization", format!("Bearer {}", token.0)))
        .uri("/gates/open/bathroom")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // open the unavailable room

    let req = test::TestRequest::post()
        .insert_header(("Authorization", format!("Bearer {}", token.0)))
        .uri("/gates/open/not_found")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn list_of_gates() {
    let app = init_test_env!();

    let token = Jwt::new(JWT_SIGN_KEY.to_string()).issue_token(
        "admin".to_string(),
        [
            Gate {
                id: 3,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 4,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 10,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 100,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
        ]
        .to_vec(),
        Duration::from_secs(60),
    );

    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {}", token.0)))
        .uri("/gates/list")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: gates::Response = test::read_body_json(resp).await;

    assert_eq!(
        &body.gates,
        &[
            Gate {
                id: 3,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 4,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 10,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
            Gate {
                id: 100,
                retries: 1,
                name: "".to_string(),
                description: "".to_string(),
            },
        ]
    );
}
