use std::collections::BTreeMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde_json::json;

use crate::{
    common::state::AppState,
    dto::dashboard::DashboardSummaryQuery,
    error::app_error::AppError,
    model::dashboard::{CashTrendPoint, DashboardSummary},
    repository::{balance_calibration_repository, dashboard_repository, user_repository},
};

pub async fn summary(
    state: &AppState,
    query: &DashboardSummaryQuery,
) -> Result<DashboardSummary, AppError> {
    validate_query(state, query).await?;
    let pool = state.db()?;
    let personal_cash =
        cash_snapshot(pool, query.user_id, query.end_date, query.start_date).await?;
    let legacy_summary =
        dashboard_repository::latest_legacy_homepage_summary(pool, query.user_id).await?;

    let income = dashboard_repository::sum_cash_bill_amounts(
        pool,
        query.user_id,
        query.start_date,
        query.end_date,
        &["income", "refund", "reduce_position", "dividend"],
    )
    .await?;
    let cash_expense = dashboard_repository::sum_cash_bill_amounts(
        pool,
        query.user_id,
        query.start_date,
        query.end_date,
        &["expense", "open_position", "add_position"],
    )
    .await?;
    let credit_card_repaid = dashboard_repository::sum_credit_card_repaid_amount(
        pool,
        query.user_id,
        query.start_date,
        query.end_date,
    )
    .await?;
    let expense = cash_expense + credit_card_repaid;
    let net_cash_flow = income - expense;
    let wealth_amount =
        dashboard_repository::sum_investments_by_type(pool, query.user_id, "wealth").await?;
    let stock_market_value =
        dashboard_repository::sum_investments_by_type(pool, query.user_id, "stock").await?;
    let family_wealth_amount = family_investment_total(pool, "wealth").await?;
    let family_stock_market_value = family_investment_total(pool, "stock").await?;
    let family_stock_idle_cash = family_stock_idle_cash(pool).await?;
    let family_cash_before_stock_idle =
        family_cash_before_stock_idle(pool, query.end_date, query.start_date).await?;
    let stock_amount = stock_market_value + personal_cash.stock_idle_cash;
    let family_stock_amount = family_stock_market_value + family_stock_idle_cash;
    let credit_card_outstanding_amount = dashboard_repository::sum_credit_card_outstanding(
        pool,
        query.user_id,
        query.start_date,
        query.end_date,
    )
    .await?;
    let debt_outstanding_amount = dashboard_repository::sum_scheduled_debt_outstanding(
        pool,
        query.user_id,
        query.start_date,
        query.end_date,
    )
    .await?;
    let outstanding_amount = credit_card_outstanding_amount + debt_outstanding_amount;
    let raw_total_assets =
        family_cash_before_stock_idle + family_wealth_amount + family_stock_market_value;
    let mut cash_balance = personal_cash.display_cash_balance;
    let mut total_assets = raw_total_assets + family_stock_idle_cash;
    let mut wealth_display = wealth_amount;
    let mut stock_display = stock_amount;
    let mut salary_prep = json!({});
    let mut position_summary = json!({});
    let mut repay_trend = json!({});
    let mut credit_cards = Vec::new();
    let mut cycle_debts = Vec::new();
    let mut pending_all = Vec::new();
    if let Some(legacy) = legacy_summary {
        cash_balance = legacy.liquid_asset;
        total_assets = legacy.family_total_asset;
        wealth_display = legacy.financing_market_value;
        stock_display = legacy.stock_total_asset;
        salary_prep = legacy
            .summary_payload
            .get("obligations")
            .cloned()
            .unwrap_or_else(|| json!({}));
        position_summary = legacy
            .summary_payload
            .get("position_summary")
            .cloned()
            .unwrap_or_else(|| json!({}));
        repay_trend = legacy
            .summary_payload
            .get("obligations")
            .and_then(|item| item.get("trend"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        credit_cards = legacy
            .summary_payload
            .get("obligations")
            .and_then(|item| item.get("credit_cards"))
            .and_then(|item| item.as_array())
            .cloned()
            .unwrap_or_default();
        cycle_debts = legacy
            .summary_payload
            .get("obligations")
            .and_then(|item| item.get("cycle_debts"))
            .and_then(|item| item.as_array())
            .cloned()
            .unwrap_or_default();
        pending_all = legacy
            .summary_payload
            .get("obligations")
            .and_then(|item| item.get("upcoming"))
            .and_then(|item| item.as_array())
            .cloned()
            .unwrap_or_default();
    }

    Ok(DashboardSummary {
        start_date: query.start_date,
        end_date: query.end_date,
        cash_balance: format_decimal2(cash_balance),
        total_assets: format_decimal2(total_assets),
        outstanding_amount: format_decimal2(outstanding_amount),
        wealth_amount: format_decimal2(wealth_display),
        stock_amount: format_decimal2(stock_display),
        family_wealth_amount: format_decimal2(family_wealth_amount),
        family_stock_amount: format_decimal2(family_stock_amount),
        stock_idle_cash: format_decimal2(personal_cash.stock_idle_cash),
        credit_card_outstanding_amount: format_decimal2(credit_card_outstanding_amount),
        debt_outstanding_amount: format_decimal2(debt_outstanding_amount),
        income: format_decimal2(income),
        expense: format_decimal2(expense),
        net_cash_flow: format_decimal2(net_cash_flow),
        calibration_status: personal_cash.calibration_status,
        calibration_date: personal_cash.calibration_date,
        salary_prep,
        position_summary,
        repay_trend,
        credit_cards,
        cycle_debts,
        pending_all,
    })
}

pub async fn cash_trend(
    state: &AppState,
    query: &DashboardSummaryQuery,
) -> Result<Vec<CashTrendPoint>, AppError> {
    validate_query(state, query).await?;
    let pool = state.db()?;
    let cash = cash_snapshot(pool, query.user_id, query.end_date, query.start_date).await?;
    let legacy_summary =
        dashboard_repository::latest_legacy_homepage_summary(pool, query.user_id).await?;
    let trend_end =
        match dashboard_repository::latest_credit_card_repay_deadline(pool, query.user_id).await? {
            Some(next_repay_date) if next_repay_date > query.end_date => next_repay_date,
            _ => query.end_date,
        };
    let trend_start = cash.calibration_date.unwrap_or(query.start_date);
    let delta_start = cash
        .calibration_date
        .and_then(|item| item.succ_opt())
        .unwrap_or(query.start_date);
    let mut balance = cash.baseline_cash;

    let mut delta_map: BTreeMap<NaiveDate, Decimal> =
        dashboard_repository::daily_index_cash_deltas(pool, query.user_id, delta_start, trend_end)
            .await?
            .into_iter()
            .collect();
    for (date, delta) in dashboard_repository::daily_debt_repayment_deltas(
        pool,
        query.user_id,
        delta_start,
        trend_end,
    )
    .await?
    {
        let entry = delta_map.entry(date).or_insert(Decimal::ZERO);
        *entry += delta;
    }

    let mut points = Vec::new();
    if cash.calibration_date.is_some() {
        points.push(CashTrendPoint {
            date: trend_start,
            cash_balance: format_decimal2(balance - cash.stock_idle_cash),
            anchor_date: cash.calibration_date,
        });
    }

    let mut cursor = if cash.calibration_date.is_some() {
        delta_start
    } else {
        trend_start
    };
    while cursor <= trend_end {
        balance += delta_map.get(&cursor).cloned().unwrap_or(Decimal::ZERO);
        points.push(CashTrendPoint {
            date: cursor,
            cash_balance: format_decimal2(balance - cash.stock_idle_cash),
            anchor_date: cash.calibration_date,
        });
        let Some(next_date) = cursor.succ_opt() else {
            break;
        };
        cursor = next_date;
    }

    if let (Some(legacy), Some(last)) = (legacy_summary, points.last())
        && let Ok(last_balance) = last.cash_balance.parse::<Decimal>()
    {
        let diff = legacy.liquid_asset - last_balance;
        if !diff.is_zero() {
            for point in &mut points {
                if let Ok(value) = point.cash_balance.parse::<Decimal>() {
                    point.cash_balance = format_decimal2(value + diff);
                }
            }
        }
    }

    Ok(points)
}

async fn validate_query(state: &AppState, query: &DashboardSummaryQuery) -> Result<(), AppError> {
    if query.start_date > query.end_date {
        return Err(AppError::BadRequest(
            "start_date must not be later than end_date".to_string(),
        ));
    }
    let _ = user_repository::find_by_id(state.db()?, query.user_id).await?;
    Ok(())
}

fn format_decimal2(value: Decimal) -> String {
    value.round_dp(2).to_string()
}

struct CashSnapshot {
    calibration_date: Option<NaiveDate>,
    calibration_status: String,
    baseline_cash: Decimal,
    balance_before_stock_idle: Decimal,
    display_cash_balance: Decimal,
    stock_idle_cash: Decimal,
}

async fn cash_snapshot(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    end_date: NaiveDate,
    fallback_start: NaiveDate,
) -> Result<CashSnapshot, AppError> {
    let calibration =
        balance_calibration_repository::latest_before_or_on(pool, user_id, end_date).await?;
    let baseline_cash = calibration
        .as_ref()
        .and_then(|item| item.cash_balance.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);
    let flow_start = calibration
        .as_ref()
        .and_then(|item| item.calibration_date.succ_opt())
        .unwrap_or(fallback_start);
    let delta =
        dashboard_repository::sum_index_cash_delta_since(pool, user_id, flow_start, end_date)
            .await?;
    let stock_idle_cash = dashboard_repository::stock_idle_cash(pool, user_id).await?;
    let balance_before_stock_idle = baseline_cash + delta;

    Ok(CashSnapshot {
        calibration_date: calibration.as_ref().map(|item| item.calibration_date),
        calibration_status: if calibration.is_some() {
            "calibrated".to_string()
        } else {
            "uncalibrated".to_string()
        },
        baseline_cash,
        balance_before_stock_idle,
        display_cash_balance: balance_before_stock_idle - stock_idle_cash,
        stock_idle_cash,
    })
}

async fn family_investment_total(
    pool: &sqlx::MySqlPool,
    investment_type: &str,
) -> Result<Decimal, AppError> {
    let mut total = Decimal::ZERO;
    for user in user_repository::list_options(pool).await? {
        if user.username == "admin" {
            continue;
        }
        total +=
            dashboard_repository::sum_investments_by_type(pool, user.id, investment_type).await?;
    }
    Ok(total)
}

async fn family_stock_idle_cash(pool: &sqlx::MySqlPool) -> Result<Decimal, AppError> {
    let mut total = Decimal::ZERO;
    for user in user_repository::list_options(pool).await? {
        if user.username == "admin" {
            continue;
        }
        total += dashboard_repository::stock_idle_cash(pool, user.id).await?;
    }
    Ok(total)
}

async fn family_cash_before_stock_idle(
    pool: &sqlx::MySqlPool,
    end_date: NaiveDate,
    fallback_start: NaiveDate,
) -> Result<Decimal, AppError> {
    let mut total = Decimal::ZERO;
    for user in user_repository::list_options(pool).await? {
        if user.username == "admin" {
            continue;
        }
        total += cash_snapshot(pool, user.id, end_date, fallback_start)
            .await?
            .balance_before_stock_idle;
    }
    Ok(total)
}
