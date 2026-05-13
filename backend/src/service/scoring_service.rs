use std::str::FromStr;

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::{
    common::state::AppState,
    dto::investment::InvestmentTopQuery,
    error::app_error::AppError,
    model::{investment::Investment, strategy_config::StrategyConfig},
    repository::{indicator_repository, investment_repository, strategy_repository},
};

#[derive(Clone)]
struct StrategyRule {
    preferred_min_score: i32,
    cooldown_days: i32,
    take_profit_rate: Decimal,
    stop_loss_rate: Decimal,
    strategy_name: String,
}

pub async fn refresh_stock_scores(state: &AppState) -> Result<usize, AppError> {
    let pool = state.db()?;
    let today = Utc::now().date_naive();
    let investments = investment_repository::list_for_scoring(pool, "stock").await?;
    let mut affected = 0usize;

    for investment in investments {
        if investment.status == "archived" {
            continue;
        }
        let strategies =
            strategy_repository::list_enabled_by_user_type(pool, investment.user_id, "stock")
                .await?;
        let rule = select_strategy(&strategies, &investment)?;
        let metrics = build_stock_metrics(&investment, &rule)?;
        indicator_repository::upsert_stock_indicator(
            pool,
            investment.id,
            investment.user_id,
            &investment.code,
            today,
            &investment.current_price,
            &metrics.price_change_rate.to_string(),
            &metrics.profit_rate.to_string(),
            &metrics.ma_bias.to_string(),
            &metrics.volume_ratio.to_string(),
            metrics.score,
            &metrics.suggestion,
            &metrics.reason,
        )
        .await?;
        affected += 1;
    }

    Ok(affected)
}

pub async fn refresh_wealth_scores(state: &AppState) -> Result<usize, AppError> {
    let pool = state.db()?;
    let today = Utc::now().date_naive();
    let investments = investment_repository::list_for_scoring(pool, "wealth").await?;
    let mut affected = 0usize;

    for investment in investments {
        if investment.status == "archived" {
            continue;
        }
        let strategies =
            strategy_repository::list_enabled_by_user_type(pool, investment.user_id, "wealth")
                .await?;
        let rule = select_strategy(&strategies, &investment)?;
        let metrics = build_wealth_metrics(&investment, &rule)?;
        indicator_repository::upsert_wealth_indicator(
            pool,
            investment.id,
            investment.user_id,
            &investment.code,
            today,
            &investment.current_price,
            &metrics.annualized_return_1d.to_string(),
            &metrics.annualized_return_7d.to_string(),
            &metrics.annualized_return_30d.to_string(),
            &metrics.drawdown_proxy.to_string(),
            &metrics.profit_rate.to_string(),
            metrics.score,
            &metrics.suggestion,
            &metrics.reason,
        )
        .await?;
        affected += 1;
    }

    Ok(affected)
}

pub async fn list_top_recommendations(
    state: &AppState,
    query: &InvestmentTopQuery,
) -> Result<Vec<crate::model::investment_recommendation::InvestmentRecommendation>, AppError> {
    indicator_repository::list_top_recommendations(state.db()?, query).await
}

struct StockMetrics {
    price_change_rate: Decimal,
    profit_rate: Decimal,
    ma_bias: Decimal,
    volume_ratio: Decimal,
    score: i32,
    suggestion: String,
    reason: String,
}

struct WealthMetrics {
    annualized_return_1d: Decimal,
    annualized_return_7d: Decimal,
    annualized_return_30d: Decimal,
    drawdown_proxy: Decimal,
    profit_rate: Decimal,
    score: i32,
    suggestion: String,
    reason: String,
}

fn build_stock_metrics(
    investment: &Investment,
    rule: &StrategyRule,
) -> Result<StockMetrics, AppError> {
    let current_price = parse_decimal(&investment.current_price)?;
    let average_cost = parse_decimal(&investment.average_cost)?;
    let profit_rate = parse_decimal(&investment.total_profit_rate)?;
    let market_value = parse_decimal(&investment.market_value)?;
    let total_cost = parse_decimal(&investment.total_cost)?;

    let price_change_rate = if average_cost > Decimal::ZERO {
        (current_price - average_cost) / average_cost
    } else {
        Decimal::ZERO
    };
    let ma_bias = (price_change_rate * Decimal::from(100u32)).round_dp(6);
    let value_factor = if total_cost > Decimal::ZERO && market_value >= total_cost {
        Decimal::new(12, 0)
    } else if total_cost > Decimal::ZERO {
        Decimal::new(-8, 0)
    } else {
        Decimal::ZERO
    };
    let trend_factor = (profit_rate * Decimal::from(140u32)).round_dp(0);
    let volume_ratio = if market_value > Decimal::new(20000, 0) {
        Decimal::new(14, 1)
    } else if market_value > Decimal::new(5000, 0) {
        Decimal::new(11, 1)
    } else {
        Decimal::ONE
    };

    let holding_days = (Utc::now().date_naive() - investment.created_at.date()).num_days();
    let mut score = 55 + decimal_to_i32(trend_factor) + decimal_to_i32(value_factor);
    if holding_days >= i64::from(rule.cooldown_days) {
        score += 6;
    } else {
        score -= 12;
    }
    if profit_rate >= Decimal::ZERO {
        score += 8;
    }
    score = score.clamp(0, 100);

    let suggestion = if investment.status != "holding" {
        "观察"
    } else if holding_days < i64::from(rule.cooldown_days) {
        "冷静期"
    } else if profit_rate >= rule.take_profit_rate {
        "减仓"
    } else if profit_rate <= -rule.stop_loss_rate {
        "卖出"
    } else if score >= rule.preferred_min_score {
        "买入"
    } else {
        "观察"
    };

    let reason = format!(
        "{}: 收益率{}%, 估算趋势偏离{}%, 冷静期{}天",
        rule.strategy_name,
        percent_string(profit_rate),
        percent_string(ma_bias / Decimal::from(100u32)),
        rule.cooldown_days
    );

    Ok(StockMetrics {
        price_change_rate: price_change_rate.round_dp(6),
        profit_rate: profit_rate.round_dp(6),
        ma_bias,
        volume_ratio,
        score,
        suggestion: suggestion.to_string(),
        reason,
    })
}

fn build_wealth_metrics(
    investment: &Investment,
    rule: &StrategyRule,
) -> Result<WealthMetrics, AppError> {
    let profit_rate = parse_decimal(&investment.total_profit_rate)?;
    let current_nav = parse_decimal(&investment.current_price)?;
    let average_cost = parse_decimal(&investment.average_cost)?;

    let annualized_return_1d = (profit_rate * Decimal::from(365u32)).round_dp(6);
    let annualized_return_7d = (profit_rate * Decimal::from(52u32)).round_dp(6);
    let annualized_return_30d = (profit_rate * Decimal::from(12u32)).round_dp(6);
    let drawdown_proxy = if profit_rate < Decimal::ZERO {
        (-profit_rate).round_dp(6)
    } else {
        Decimal::ZERO
    };
    let nav_bias = if average_cost > Decimal::ZERO {
        ((current_nav - average_cost) / average_cost * Decimal::from(100u32)).round_dp(0)
    } else {
        Decimal::ZERO
    };
    let holding_days = (Utc::now().date_naive() - investment.created_at.date()).num_days();

    let mut score = 58 + decimal_to_i32(annualized_return_30d * Decimal::from(20u32));
    score -= decimal_to_i32(drawdown_proxy * Decimal::from(120u32));
    score += decimal_to_i32(nav_bias / Decimal::from(3u32));
    if holding_days >= i64::from(rule.cooldown_days) {
        score += 4;
    } else {
        score -= 10;
    }
    score = score.clamp(0, 100);

    let suggestion =
        if investment.status != "holding" || holding_days < i64::from(rule.cooldown_days) {
            "观望"
        } else if profit_rate >= rule.take_profit_rate {
            "止盈"
        } else if score >= rule.preferred_min_score {
            "定投"
        } else if score >= 50 {
            "持有"
        } else {
            "观望"
        };

    let reason = format!(
        "{}: 持仓收益率{}%, 7日年化代理{}%, 回撤代理{}%",
        rule.strategy_name,
        percent_string(profit_rate),
        percent_string(annualized_return_7d),
        percent_string(drawdown_proxy)
    );

    Ok(WealthMetrics {
        annualized_return_1d,
        annualized_return_7d,
        annualized_return_30d,
        drawdown_proxy,
        profit_rate: profit_rate.round_dp(6),
        score,
        suggestion: suggestion.to_string(),
        reason,
    })
}

fn select_strategy(
    strategies: &[StrategyConfig],
    investment: &Investment,
) -> Result<StrategyRule, AppError> {
    let selected = strategies
        .iter()
        .find(|item| item.target_code.as_deref() == Some(investment.code.as_str()))
        .or_else(|| strategies.iter().find(|item| item.target_code.is_none()));

    if let Some(strategy) = selected {
        return Ok(StrategyRule {
            preferred_min_score: strategy.preferred_min_score,
            cooldown_days: strategy.cooldown_days,
            take_profit_rate: parse_decimal(&strategy.take_profit_rate)?,
            stop_loss_rate: parse_decimal(&strategy.stop_loss_rate)?,
            strategy_name: strategy.strategy_name.clone(),
        });
    }

    Ok(StrategyRule {
        preferred_min_score: if investment.investment_type == "stock" {
            68
        } else {
            62
        },
        cooldown_days: 3,
        take_profit_rate: if investment.investment_type == "stock" {
            Decimal::new(15, 2)
        } else {
            Decimal::new(8, 2)
        },
        stop_loss_rate: Decimal::new(8, 2),
        strategy_name: "默认策略".to_string(),
    })
}

fn parse_decimal(value: &str) -> Result<Decimal, AppError> {
    Decimal::from_str(value).map_err(|_| AppError::internal_with_log("invalid decimal value"))
}

fn decimal_to_i32(value: Decimal) -> i32 {
    value.trunc().to_i32().unwrap_or_default()
}

fn percent_string(value: Decimal) -> String {
    (value * Decimal::from(100u32)).round_dp(2).to_string()
}
