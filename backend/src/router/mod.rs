use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    common::{auth, state::AppState},
    handler,
};

pub fn create_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/auth/logout", post(handler::auth::logout))
        .route("/api/auth/me", get(handler::auth::me))
        .route(
            "/api/balance-calibrations",
            get(handler::balance_calibrations::list).post(handler::balance_calibrations::create),
        )
        .route(
            "/api/balance-calibrations/{id}",
            get(handler::balance_calibrations::detail)
                .put(handler::balance_calibrations::update)
                .delete(handler::balance_calibrations::delete),
        )
        .route("/api/dashboard/summary", get(handler::dashboard::summary))
        .route(
            "/api/dashboard/cash-trend",
            get(handler::dashboard::cash_trend),
        )
        .route("/api/jobs", get(handler::jobs::list))
        .route("/api/jobs/runs", get(handler::jobs::runs))
        .route(
            "/api/jobs/{id}",
            get(handler::jobs::detail).put(handler::jobs::update),
        )
        .route("/api/jobs/{id}/trigger", post(handler::jobs::trigger))
        .route(
            "/api/budgets",
            get(handler::budgets::list).post(handler::budgets::create),
        )
        .route("/api/budgets/generate", post(handler::budgets::generate))
        .route(
            "/api/budgets/{id}",
            get(handler::budgets::detail)
                .put(handler::budgets::update)
                .delete(handler::budgets::delete),
        )
        .route(
            "/api/brands",
            get(handler::brands::list).post(handler::brands::create),
        )
        .route(
            "/api/brands/{id}",
            get(handler::brands::detail)
                .put(handler::brands::update)
                .delete(handler::brands::delete),
        )
        .route(
            "/api/bills",
            get(handler::bills::list).post(handler::bills::create),
        )
        .route(
            "/api/bill-tags",
            get(handler::bill_tags::list).post(handler::bill_tags::create),
        )
        .route(
            "/api/bill-tags/{id}",
            axum::routing::put(handler::bill_tags::update).delete(handler::bill_tags::delete),
        )
        .route("/api/bills/options", get(handler::bills::options))
        .route(
            "/api/bills/{id}",
            get(handler::bills::detail)
                .put(handler::bills::update)
                .delete(handler::bills::delete),
        )
        .route(
            "/api/debts",
            get(handler::debts::list).post(handler::debts::create),
        )
        .route(
            "/api/debts/{id}",
            get(handler::debts::detail)
                .put(handler::debts::update)
                .delete(handler::debts::delete),
        )
        .route(
            "/api/presales",
            get(handler::presales::list).post(handler::presales::create),
        )
        .route(
            "/api/presales/{id}",
            get(handler::presales::detail)
                .put(handler::presales::update)
                .delete(handler::presales::delete),
        )
        .route(
            "/api/assets",
            get(handler::assets::list).post(handler::assets::create),
        )
        .route(
            "/api/assets/{id}",
            get(handler::assets::detail)
                .put(handler::assets::update)
                .delete(handler::assets::delete),
        )
        .route("/api/investments", get(handler::investments::list))
        .route("/api/investments/top", get(handler::investments::top))
        .route("/api/investments/{id}", get(handler::investments::detail))
        .route(
            "/api/investment-transactions",
            get(handler::investment_transactions::list),
        )
        .route(
            "/api/strategies",
            get(handler::strategies::list).post(handler::strategies::create),
        )
        .route(
            "/api/strategies/{id}",
            get(handler::strategies::detail)
                .put(handler::strategies::update)
                .delete(handler::strategies::delete),
        )
        .route(
            "/api/intel",
            get(handler::intel::list).post(handler::intel::create),
        )
        .route(
            "/api/migration-audit/summary",
            get(handler::migration_audit::summary),
        )
        .route(
            "/api/intel/{id}",
            get(handler::intel::detail)
                .put(handler::intel::update)
                .delete(handler::intel::delete),
        )
        .route(
            "/api/users",
            get(handler::users::list).post(handler::users::create),
        )
        .route("/api/users/options", get(handler::users::options))
        .route(
            "/api/users/{id}",
            get(handler::users::detail)
                .put(handler::users::update)
                .delete(handler::users::delete),
        )
        .route(
            "/api/config/items",
            get(handler::config_items::list).post(handler::config_items::create),
        )
        .route("/api/config/types", get(handler::config_items::list_types))
        .route(
            "/api/config/items/{id}",
            get(handler::config_items::detail)
                .put(handler::config_items::update)
                .delete(handler::config_items::delete),
        )
        .route(
            "/api/config/credit-cards",
            get(handler::credit_cards::list).post(handler::credit_cards::create),
        )
        .route(
            "/api/config/credit-cards/{id}",
            get(handler::credit_cards::detail)
                .put(handler::credit_cards::update)
                .delete(handler::credit_cards::delete),
        );

    Router::new()
        .route("/api/health", get(handler::health::health))
        .route("/api/auth/captcha", get(handler::auth::captcha))
        .route("/api/auth/login", post(handler::auth::login))
        .merge(protected.route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        )))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
