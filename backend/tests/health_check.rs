use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hfs_backend::{common::state::AppState, config::settings::Settings, router};
use tower::util::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_success() {
    let state = AppState::new(Settings::default(), None, None);
    let app = router::create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
