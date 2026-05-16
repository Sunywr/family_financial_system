use std::collections::BTreeMap;

use chrono::{Datelike, Duration, Local, NaiveDate};
use rust_decimal::Decimal;
use serde_json::Value;

use crate::{
    common::state::AppState,
    dto::dashboard::DashboardSummaryQuery,
    error::app_error::AppError,
    model::dashboard::{
        CashTrendMarkPoint, CashTrendPoint, DashboardPendingItem, DashboardPositionSummary,
        DashboardRepayTrendItem, DashboardRepayTrendSummary, DashboardSalaryPrepSummary,
        DashboardSummary,
    },
    repository::{
        balance_calibration_repository, config_item_repository, dashboard_repository,
        user_repository,
    },
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
    let has_legacy_summary = legacy_summary.is_some();

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
    let investment_overview =
        dashboard_repository::investment_overview(pool, query.user_id).await?;
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

    let salary_samples = dashboard_repository::list_recent_salary_income_days(
        pool,
        query.user_id,
        query.end_date - Duration::days(365),
        query.end_date,
    )
    .await?;
    let salary_day = resolve_salary_day(&salary_samples);
    let salary_target_date = resolve_salary_target_date(query.end_date, salary_day);
    let next_salary_date = resolve_salary_target_date(salary_target_date, salary_day);
    let upcoming_end = next_salary_date;
    let pending_debts = dashboard_repository::list_pending_debts_upcoming(
        pool,
        query.user_id,
        query.end_date,
        upcoming_end,
        20,
    )
    .await?;
    let pending_cycle_bills = dashboard_repository::list_cycle_debt_bills_upcoming(
        pool,
        query.user_id,
        query.end_date,
        upcoming_end,
        200,
    )
    .await?;
    let credit_pending_items = pending_debts
        .iter()
        .filter(|item| item.payment_method == "credit_card")
        .cloned()
        .collect::<Vec<_>>();
    let debt_cycle_items = pending_debts
        .iter()
        .filter(|item| item.payment_method != "credit_card")
        .cloned()
        .collect::<Vec<_>>();
    let cycle_pending_items =
        merge_cycle_pending_items(&debt_cycle_items, &pending_cycle_bills, query.end_date);

    let mut pending_credit_total = Decimal::ZERO;
    let mut pending_cycle_total = Decimal::ZERO;
    let mut due_before_salary_credit_total = Decimal::ZERO;
    let mut due_before_salary_cycle_total = Decimal::ZERO;
    let mut pending_credit_count = 0u64;
    let mut pending_cycle_count = 0u64;
    let mut credit_cards = Vec::new();
    let mut cycle_debts = Vec::new();
    let mut pending_all = Vec::new();
    let mut pending_by_date: BTreeMap<NaiveDate, (Decimal, Decimal, Vec<String>, Vec<String>)> =
        BTreeMap::new();

    for item in &credit_pending_items {
        let row = DashboardPendingItem {
            id: Some(item.id),
            name: item.display_name.clone(),
            item_type: pending_item_type(&item.payment_method).to_string(),
            due_date: item.repay_deadline,
            amount: format_decimal2(item.amount),
        };
        pending_all.push(row.clone());

        let entry = pending_by_date.entry(item.repay_deadline).or_insert((
            Decimal::ZERO,
            Decimal::ZERO,
            Vec::new(),
            Vec::new(),
        ));
        pending_credit_total += item.amount;
        pending_credit_count += 1;
        if item.repay_deadline <= salary_target_date {
            due_before_salary_credit_total += item.amount;
        }
        entry.0 += item.amount;
        entry.2.push(item.display_name.clone());
        credit_cards.push(row);
    }

    for item in &cycle_pending_items {
        let row = DashboardPendingItem {
            id: Some(item.id),
            name: item.display_name.clone(),
            item_type: pending_item_type(&item.payment_method).to_string(),
            due_date: item.repay_deadline,
            amount: format_decimal2(item.amount),
        };
        pending_all.push(row.clone());

        let entry = pending_by_date.entry(item.repay_deadline).or_insert((
            Decimal::ZERO,
            Decimal::ZERO,
            Vec::new(),
            Vec::new(),
        ));

        pending_cycle_total += item.amount;
        pending_cycle_count += 1;
        if item.repay_deadline <= salary_target_date {
            due_before_salary_cycle_total += item.amount;
        }
        entry.1 += item.amount;
        entry.3.push(item.display_name.clone());
        cycle_debts.push(row);
    }

    pending_all.sort_by_key(|item| item.due_date);

    let trend_rows = pending_by_date
        .into_iter()
        .map(
            |(date, (credit_total, cycle_total, credit_names, cycle_names))| {
                let total = credit_total + cycle_total;
                DashboardRepayTrendItem {
                    date,
                    credit_due: format_decimal2(credit_total),
                    cycle_due: format_decimal2(cycle_total),
                    credit_names: credit_names.join("、"),
                    cycle_names: cycle_names.join("、"),
                    total_due: format_decimal2(total),
                    before_salary: date <= salary_target_date,
                }
            },
        )
        .collect::<Vec<_>>();

    if let Some(legacy) = legacy_summary {
        cash_balance = legacy.liquid_asset;
        total_assets = legacy.family_total_asset;
        wealth_display = legacy.financing_market_value;
        stock_display = legacy.stock_total_asset;

        if credit_cards.is_empty() {
            credit_cards = parse_pending_items(
                legacy
                    .summary_payload
                    .get("obligations")
                    .and_then(|item| item.get("credit_cards"))
                    .and_then(|item| item.as_array()),
                "信用卡",
            );
        }
        if cycle_debts.is_empty() {
            cycle_debts = parse_pending_items(
                legacy
                    .summary_payload
                    .get("obligations")
                    .and_then(|item| item.get("cycle_debts"))
                    .and_then(|item| item.as_array()),
                "周期债务",
            );
        }
        if pending_all.is_empty() {
            pending_all = parse_pending_items(
                legacy
                    .summary_payload
                    .get("obligations")
                    .and_then(|item| item.get("upcoming"))
                    .and_then(|item| item.as_array()),
                "待处理项",
            );
        }
    }

    let salary_prep = DashboardSalaryPrepSummary {
        window_start: query.end_date,
        window_end: upcoming_end,
        salary_day,
        salary_target_date,
        days_until_salary: (salary_target_date - query.end_date).num_days().max(0),
        pending_count: pending_all.len(),
        credit_count: pending_credit_count,
        cycle_count: pending_cycle_count,
        credit_due_before_salary: format_decimal2(due_before_salary_credit_total),
        cycle_due_before_salary: format_decimal2(due_before_salary_cycle_total),
        due_before_salary: format_decimal2(
            due_before_salary_credit_total + due_before_salary_cycle_total,
        ),
        credit_due_in_window: format_decimal2(pending_credit_total),
        cycle_due_in_window: format_decimal2(pending_cycle_total),
        source: "ffs_realtime".to_string(),
    };

    let repay_trend = DashboardRepayTrendSummary {
        source: "ffs_realtime".to_string(),
        window_start: query.end_date,
        window_end: upcoming_end,
        salary_target_date,
        items: trend_rows,
        before_salary_total: format_decimal2(
            due_before_salary_credit_total + due_before_salary_cycle_total,
        ),
        window_total: format_decimal2(pending_credit_total + pending_cycle_total),
    };

    let position_summary = DashboardPositionSummary {
        personal_wealth: format_decimal2(wealth_display),
        personal_stock: format_decimal2(stock_display),
        total_investment: format_decimal2(investment_overview.total_investment),
        holding_profit: format_decimal2(investment_overview.holding_profit),
        total_profit: format_decimal2(investment_overview.total_profit),
        avg_profit_rate: format_percent2(investment_overview.avg_profit_rate),
        avg_annual_rate_wealth: format_percent2(investment_overview.avg_annual_rate_wealth),
        family_wealth: format_decimal2(family_wealth_amount),
        family_stock: format_decimal2(family_stock_amount),
        stock_idle_cash: format_decimal2(personal_cash.stock_idle_cash),
        source: if has_legacy_summary {
            "legacy_snapshot".to_string()
        } else {
            "ffs_realtime".to_string()
        },
    };

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

    let today = Local::now().date_naive();
    let projection_start = match today.succ_opt() {
        Some(tomorrow) if tomorrow > delta_start => tomorrow,
        _ => delta_start,
    };
    let mut mark_events_by_date: BTreeMap<NaiveDate, Vec<(u64, String, String, Decimal)>> =
        BTreeMap::new();
    if projection_start <= trend_end {
        let use_provident_fund_for_mortgage = config_item_repository::find_by_type_and_name(
            pool,
            "system_setting",
            "use_provident_fund_for_mortgage",
        )
        .await?
        .map(|item| item.enabled)
        .unwrap_or(false);
        let pending_debts = dashboard_repository::list_pending_debts_upcoming(
            pool,
            query.user_id,
            projection_start,
            trend_end,
            1000,
        )
        .await?;

        for debt in pending_debts {
            if use_provident_fund_for_mortgage && is_mortgage_like(&debt) {
                continue;
            }
            let debt_type = pending_item_type(&debt.payment_method).to_string();
            let entry = delta_map
                .entry(debt.repay_deadline)
                .or_insert(Decimal::ZERO);
            *entry -= debt.amount;
            mark_events_by_date
                .entry(debt.repay_deadline)
                .or_default()
                .push((debt.id, debt.display_name, debt_type, debt.amount));
        }
    }

    let mut points = Vec::new();
    if cash.calibration_date.is_some() {
        points.push(CashTrendPoint {
            date: trend_start,
            cash_balance: format_decimal2(balance - cash.stock_idle_cash),
            anchor_date: cash.calibration_date,
            mark_points: Vec::new(),
        });
    }

    let mut cursor = if cash.calibration_date.is_some() {
        delta_start
    } else {
        trend_start
    };
    while cursor <= trend_end {
        balance += delta_map.get(&cursor).cloned().unwrap_or(Decimal::ZERO);
        let cash_balance = format_decimal2(balance - cash.stock_idle_cash);
        let mark_points = mark_events_by_date
            .get(&cursor)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(
                |(debt_id, debt_name, debt_type, amount)| CashTrendMarkPoint {
                    debt_id,
                    debt_name,
                    debt_type,
                    due_date: cursor,
                    amount: format_decimal2(amount),
                    balance_after: cash_balance.clone(),
                },
            )
            .collect::<Vec<_>>();
        points.push(CashTrendPoint {
            date: cursor,
            cash_balance,
            anchor_date: cash.calibration_date,
            mark_points,
        });
        let Some(next_date) = cursor.succ_opt() else {
            break;
        };
        cursor = next_date;
    }

    if let Some(legacy) = legacy_summary {
        let align_date = if today < trend_start {
            trend_start
        } else if today > trend_end {
            trend_end
        } else {
            today
        };
        let align_balance = points
            .iter()
            .find(|point| point.date == align_date)
            .or_else(|| points.last())
            .and_then(|point| point.cash_balance.parse::<Decimal>().ok());

        if let Some(current_balance) = align_balance {
            let diff = legacy.liquid_asset - current_balance;
            if !diff.is_zero() {
                for point in &mut points {
                    if let Ok(value) = point.cash_balance.parse::<Decimal>() {
                        point.cash_balance = format_decimal2(value + diff);
                    }
                }
            }
        }

        if let Some(point) = points.iter_mut().find(|point| point.date == align_date) {
            point.cash_balance = format_decimal2(legacy.liquid_asset);
        }
    }

    let points = points
        .into_iter()
        .filter(|point| point.date >= query.start_date)
        .collect::<Vec<_>>();

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

fn pending_item_type(payment_method: &str) -> &'static str {
    if payment_method == "credit_card" {
        "信用卡"
    } else {
        "周期债务"
    }
}

fn parse_pending_items(
    values: Option<&Vec<Value>>,
    default_type: &str,
) -> Vec<DashboardPendingItem> {
    values
        .into_iter()
        .flat_map(|items| items.iter())
        .filter_map(|item| parse_pending_item(item, default_type))
        .collect()
}

fn parse_pending_item(value: &Value, default_type: &str) -> Option<DashboardPendingItem> {
    let due_date = value
        .get("due_date")
        .and_then(Value::as_str)
        .and_then(|raw| NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok())?;

    let name = value
        .get("name")
        .and_then(Value::as_str)?
        .trim()
        .to_string();
    if name.is_empty() {
        return None;
    }

    Some(DashboardPendingItem {
        id: value.get("id").and_then(Value::as_u64),
        name,
        item_type: value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or(default_type)
            .to_string(),
        due_date,
        amount: value
            .get("amount")
            .and_then(Value::as_str)
            .unwrap_or("--")
            .to_string(),
    })
}

fn merge_cycle_pending_items(
    debt_projection_items: &[crate::repository::dashboard_repository::PendingDebtItem],
    billed_items: &[crate::repository::dashboard_repository::PendingDebtItem],
    boundary_date: NaiveDate,
) -> Vec<crate::repository::dashboard_repository::PendingDebtItem> {
    if billed_items.is_empty() {
        return debt_projection_items.to_vec();
    }

    let mut merged = billed_items
        .iter()
        .filter(|item| item.repay_deadline <= boundary_date)
        .cloned()
        .collect::<Vec<_>>();
    merged.extend(
        debt_projection_items
            .iter()
            .filter(|item| item.repay_deadline > boundary_date)
            .cloned(),
    );
    merged.sort_by_key(|item| (item.repay_deadline, item.id));
    merged
}

fn format_decimal2(value: Decimal) -> String {
    value.round_dp(2).to_string()
}

fn format_percent2(rate: Decimal) -> String {
    format!("{}%", (rate * Decimal::from(100u32)).round_dp(2))
}

fn is_mortgage_like(item: &crate::repository::dashboard_repository::PendingDebtItem) -> bool {
    let text = format!("{} {}", item.category_name, item.display_name).to_lowercase();
    text.contains("房贷")
        || text.contains("mortgage")
        || (text.contains("住房") && text.contains('贷'))
}

fn resolve_salary_day(days: &[u32]) -> u32 {
    if days.is_empty() {
        return 5;
    }
    let mut count_by_day: BTreeMap<u32, u32> = BTreeMap::new();
    for day in days {
        if *day == 0 || *day > 31 {
            continue;
        }
        let entry = count_by_day.entry(*day).or_insert(0);
        *entry += 1;
    }
    count_by_day
        .into_iter()
        .max_by_key(|(day, count)| (*count, u32::MAX - *day))
        .map(|(day, _)| day)
        .unwrap_or(5)
}

fn resolve_salary_target_date(anchor: NaiveDate, salary_day: u32) -> NaiveDate {
    let current_month_target = day_in_month(anchor.year(), anchor.month(), salary_day);
    if anchor.day() < current_month_target {
        return NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), current_month_target)
            .expect("valid salary date");
    }

    let (year, month) = if anchor.month() == 12 {
        (anchor.year() + 1, 1)
    } else {
        (anchor.year(), anchor.month() + 1)
    };
    let day = day_in_month(year, month, salary_day);
    NaiveDate::from_ymd_opt(year, month, day).expect("valid next salary date")
}

fn day_in_month(year: i32, month: u32, desired_day: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("valid month");
    let last_day = (next_month - Duration::days(1)).day();
    desired_day.min(last_day).max(1)
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

#[cfg(test)]
mod tests {
    use super::merge_cycle_pending_items;
    use crate::repository::dashboard_repository::PendingDebtItem;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn item(id: u64, due_date: NaiveDate, amount: i64, name: &str) -> PendingDebtItem {
        PendingDebtItem {
            id,
            category_name: "loan".to_string(),
            display_name: name.to_string(),
            payment_method: "cash".to_string(),
            amount: Decimal::from(amount),
            repay_deadline: due_date,
        }
    }

    #[test]
    fn merge_uses_projection_when_no_billed_items() {
        let boundary = NaiveDate::from_ymd_opt(2026, 5, 16).expect("valid");
        let projected = vec![
            item(
                1,
                NaiveDate::from_ymd_opt(2026, 5, 20).expect("valid"),
                100,
                "A",
            ),
            item(
                2,
                NaiveDate::from_ymd_opt(2026, 5, 25).expect("valid"),
                200,
                "B",
            ),
        ];

        let merged = merge_cycle_pending_items(&projected, &[], boundary);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].id, 1);
        assert_eq!(merged[1].id, 2);
    }

    #[test]
    fn merge_keeps_only_billed_before_or_on_boundary() {
        let boundary = NaiveDate::from_ymd_opt(2026, 5, 16).expect("valid");
        let projected = vec![
            item(
                9,
                NaiveDate::from_ymd_opt(2026, 5, 14).expect("valid"),
                999,
                "old-projected",
            ),
            item(
                10,
                NaiveDate::from_ymd_opt(2026, 5, 18).expect("valid"),
                300,
                "future-projected",
            ),
        ];
        let billed = vec![
            item(
                3,
                NaiveDate::from_ymd_opt(2026, 5, 15).expect("valid"),
                120,
                "billed-15",
            ),
            item(
                4,
                NaiveDate::from_ymd_opt(2026, 5, 16).expect("valid"),
                130,
                "billed-16",
            ),
            item(
                5,
                NaiveDate::from_ymd_opt(2026, 5, 17).expect("valid"),
                140,
                "billed-17",
            ),
        ];

        let merged = merge_cycle_pending_items(&projected, &billed, boundary);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].id, 3);
        assert_eq!(merged[1].id, 4);
        assert_eq!(merged[2].id, 10);
    }

    #[test]
    fn merge_sorts_by_date_and_id() {
        let boundary = NaiveDate::from_ymd_opt(2026, 5, 16).expect("valid");
        let projected = vec![
            item(
                7,
                NaiveDate::from_ymd_opt(2026, 5, 20).expect("valid"),
                100,
                "p-7",
            ),
            item(
                6,
                NaiveDate::from_ymd_opt(2026, 5, 20).expect("valid"),
                100,
                "p-6",
            ),
        ];
        let billed = vec![item(
            8,
            NaiveDate::from_ymd_opt(2026, 5, 16).expect("valid"),
            100,
            "b-8",
        )];

        let merged = merge_cycle_pending_items(&projected, &billed, boundary);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].id, 8);
        assert_eq!(merged[1].id, 6);
        assert_eq!(merged[2].id, 7);
    }
}
