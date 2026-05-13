use std::str::FromStr;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::{
    common::state::AppState,
    dto::{
        asset::CreateAssetRequest,
        bill::{BillListQuery, BillOptionsDto, CreateBillRequest, UpdateBillRequest},
        debt::CreateDebtRequest,
        presale::CreatePresaleRequest,
    },
    error::app_error::AppError,
    model::{bill::Bill, config_item::ConfigItem},
    repository::{
        asset_repository, bill_repository, bill_tag_repository, config_item_repository,
        credit_card_repository, debt_repository, investment_repository,
        investment_transaction_repository, presale_repository, user_repository,
    },
};

const PAYMENT_METHODS: &[&str] = &["cash", "credit_card", "installment", "presale"];
const NORMAL_BILL_TYPES: &[&str] = &["income", "expense", "refund"];
const INVESTMENT_BILL_TYPES: &[&str] = &[
    "open_position",
    "add_position",
    "reduce_position",
    "dividend",
];
const TRANSFER_TARGET_TYPES: &[&str] = &["system_user", "other_person"];

pub async fn list(state: &AppState, query: &BillListQuery) -> Result<(Vec<Bill>, u64), AppError> {
    bill_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<Bill, AppError> {
    bill_repository::find_by_id(state.db()?, id).await
}

pub async fn create(state: &AppState, payload: &CreateBillRequest) -> Result<Bill, AppError> {
    let pool = state.db()?;
    let category = validate_create_payload(state, payload).await?;
    ensure_tags_exist(state, payload.user_id, payload.tags.as_deref()).await?;
    let special_status = derive_special_status(&category, payload);

    if is_transfer_category(&category)
        && payload.transfer_target_type.as_deref() == Some("system_user")
    {
        let target_user_id = payload.transfer_target_user_id.ok_or_else(|| {
            AppError::BadRequest(
                "transfer_target_user_id is required for system_user transfer".to_string(),
            )
        })?;
        let transfer_group_id = format!(
            "transfer-{}-{}-{}",
            payload.user_id,
            target_user_id,
            chrono::Utc::now().timestamp_millis()
        );
        let id = bill_repository::create(
            pool,
            payload,
            &category.display_name,
            Some(&transfer_group_id),
            &special_status,
        )
        .await?;
        bill_repository::create_transfer_mirror(
            pool,
            payload,
            target_user_id,
            &category.display_name,
            &transfer_group_id,
        )
        .await?;
        return detail(state, id).await;
    }

    let id = bill_repository::create(pool, payload, &category.display_name, None, &special_status)
        .await?;
    handle_bill_side_effects(state, id, &category, payload).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateBillRequest,
) -> Result<Bill, AppError> {
    let category = validate_update_payload(state, payload).await?;
    let bill = bill_repository::find_by_id(state.db()?, id).await?;
    ensure_tags_exist(state, bill.user_id, payload.tags.as_deref()).await?;
    let special_status = derive_special_status(&category, payload);
    bill_repository::update(
        state.db()?,
        id,
        payload,
        &category.display_name,
        &special_status,
    )
    .await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    bill_repository::soft_delete(state.db()?, id).await
}

pub fn options() -> BillOptionsDto {
    BillOptionsDto {
        payment_methods: PAYMENT_METHODS,
        normal_bill_types: NORMAL_BILL_TYPES,
        investment_bill_types: INVESTMENT_BILL_TYPES,
        transfer_target_types: TRANSFER_TARGET_TYPES,
    }
}

async fn validate_create_payload(
    state: &AppState,
    payload: &CreateBillRequest,
) -> Result<ConfigItem, AppError> {
    validate_common(
        state,
        payload.user_id,
        payload.category_id,
        &payload.bill_type,
        &payload.payment_method,
    )
    .await?;
    validate_amount(&payload.amount)?;
    validate_tags(payload.tags.as_deref())?;
    validate_special_fields_create(state, payload).await
}

async fn validate_update_payload(
    state: &AppState,
    payload: &UpdateBillRequest,
) -> Result<ConfigItem, AppError> {
    validate_common_for_update(
        state,
        payload.category_id,
        &payload.bill_type,
        &payload.payment_method,
    )
    .await?;
    validate_amount(&payload.amount)?;
    validate_tags(payload.tags.as_deref())?;
    validate_special_fields_update(state, payload).await
}

async fn validate_common(
    state: &AppState,
    user_id: u64,
    category_id: u64,
    bill_type: &str,
    payment_method: &str,
) -> Result<(), AppError> {
    let _ = user_repository::find_by_id(state.db()?, user_id).await?;
    let _ = config_item_repository::find_by_id(state.db()?, category_id).await?;
    validate_bill_type(bill_type)?;
    validate_payment_method(payment_method)?;
    Ok(())
}

async fn validate_common_for_update(
    state: &AppState,
    category_id: u64,
    bill_type: &str,
    payment_method: &str,
) -> Result<(), AppError> {
    let _ = config_item_repository::find_by_id(state.db()?, category_id).await?;
    validate_bill_type(bill_type)?;
    validate_payment_method(payment_method)?;
    Ok(())
}

async fn validate_special_fields_create(
    state: &AppState,
    payload: &CreateBillRequest,
) -> Result<ConfigItem, AppError> {
    let pool = state.db()?;
    let category = config_item_repository::find_by_id(pool, payload.category_id).await?;
    validate_special_fields(
        pool,
        &category,
        &payload.payment_method,
        &payload.bill_type,
        payload.transfer_target_type.as_deref(),
        payload.transfer_target_user_id,
        payload.credit_card_id,
        payload.is_installment.unwrap_or(false),
        payload.installment_months,
        payload.investment_action.as_deref(),
        payload.product_code.as_deref(),
        payload.product_name.as_deref(),
        payload.organization_name.as_deref(),
        payload.share_amount.as_deref(),
        payload.is_fixed_asset,
    )
    .await?;
    Ok(category)
}

#[allow(clippy::too_many_arguments)]
async fn validate_special_fields_update(
    state: &AppState,
    payload: &UpdateBillRequest,
) -> Result<ConfigItem, AppError> {
    let pool = state.db()?;
    let category = config_item_repository::find_by_id(pool, payload.category_id).await?;
    validate_special_fields(
        pool,
        &category,
        &payload.payment_method,
        &payload.bill_type,
        payload.transfer_target_type.as_deref(),
        payload.transfer_target_user_id,
        payload.credit_card_id,
        payload.is_installment.unwrap_or(false),
        payload.installment_months,
        payload.investment_action.as_deref(),
        payload.product_code.as_deref(),
        payload.product_name.as_deref(),
        payload.organization_name.as_deref(),
        payload.share_amount.as_deref(),
        payload.is_fixed_asset,
    )
    .await?;
    Ok(category)
}

#[allow(clippy::too_many_arguments)]
async fn validate_special_fields(
    pool: &sqlx::MySqlPool,
    category: &ConfigItem,
    payment_method: &str,
    bill_type: &str,
    transfer_target_type: Option<&str>,
    transfer_target_user_id: Option<u64>,
    credit_card_id: Option<u64>,
    is_installment: bool,
    installment_months: Option<u32>,
    investment_action: Option<&str>,
    product_code: Option<&str>,
    product_name: Option<&str>,
    organization_name: Option<&str>,
    share_amount: Option<&str>,
    is_fixed_asset: bool,
) -> Result<(), AppError> {
    if is_investment_category(category) {
        if payment_method != "cash" {
            return Err(AppError::BadRequest(
                "investment category bills must use cash payment_method".to_string(),
            ));
        }
        if !INVESTMENT_BILL_TYPES.contains(&bill_type) {
            return Err(AppError::BadRequest(
                "investment category must use investment bill_type".to_string(),
            ));
        }
        if investment_action.unwrap_or_default().trim().is_empty()
            || product_code.unwrap_or_default().trim().is_empty()
            || product_name.unwrap_or_default().trim().is_empty()
            || organization_name.unwrap_or_default().trim().is_empty()
            || share_amount.unwrap_or_default().trim().is_empty()
        {
            return Err(AppError::BadRequest(
                "investment bills require investment_action, product_code, product_name, organization_name and share_amount".to_string(),
            ));
        }
        validate_decimal(share_amount.unwrap_or_default(), "share_amount")?;
    }

    if is_transfer_category(category) {
        if payment_method != "cash" {
            return Err(AppError::BadRequest(
                "transfer bills must use cash payment_method".to_string(),
            ));
        }
        if transfer_target_type.is_none() {
            return Err(AppError::BadRequest(
                "transfer_target_type is required for transfer bills".to_string(),
            ));
        }
        if let Some(target_type) = transfer_target_type {
            if !TRANSFER_TARGET_TYPES.contains(&target_type) {
                return Err(AppError::BadRequest(
                    "unsupported transfer_target_type".to_string(),
                ));
            }
            if target_type == "system_user" {
                let target_user_id = transfer_target_user_id.ok_or_else(|| {
                    AppError::BadRequest("transfer_target_user_id is required".to_string())
                })?;
                let _ = user_repository::find_by_id(pool, target_user_id).await?;
            }
        }
    }

    if payment_method == "credit_card" {
        let credit_card_id = credit_card_id.ok_or_else(|| {
            AppError::BadRequest(
                "credit_card_id is required when payment_method is credit_card".to_string(),
            )
        })?;
        let _ = credit_card_repository::find_by_id(pool, credit_card_id).await?;
    }

    if is_installment {
        if payment_method != "credit_card" && payment_method != "installment" {
            return Err(AppError::BadRequest(
                "installment bills must use credit_card or installment payment_method".to_string(),
            ));
        }
        if installment_months.unwrap_or(0) == 0 {
            return Err(AppError::BadRequest(
                "installment_months is required when is_installment is true".to_string(),
            ));
        }
    }

    if is_fixed_asset && is_investment_category(category) {
        return Err(AppError::BadRequest(
            "investment bills cannot also be marked as fixed_asset".to_string(),
        ));
    }

    Ok(())
}

fn validate_payment_method(payment_method: &str) -> Result<(), AppError> {
    if !PAYMENT_METHODS.contains(&payment_method) {
        return Err(AppError::BadRequest(
            "unsupported payment_method".to_string(),
        ));
    }
    Ok(())
}

fn validate_bill_type(bill_type: &str) -> Result<(), AppError> {
    if !NORMAL_BILL_TYPES.contains(&bill_type) && !INVESTMENT_BILL_TYPES.contains(&bill_type) {
        return Err(AppError::BadRequest("unsupported bill_type".to_string()));
    }
    Ok(())
}

fn validate_amount(value: &str) -> Result<(), AppError> {
    validate_decimal(value, "amount")
}

fn validate_decimal(value: &str, field: &str) -> Result<(), AppError> {
    let decimal = Decimal::from_str(value)
        .map_err(|_| AppError::BadRequest(format!("{field} must be a valid decimal string")))?;
    if decimal <= Decimal::ZERO {
        return Err(AppError::BadRequest(format!(
            "{field} must be greater than zero"
        )));
    }
    Ok(())
}

fn validate_tags(tags: Option<&[String]>) -> Result<(), AppError> {
    if let Some(tags) = tags {
        if tags.len() > 20 {
            return Err(AppError::BadRequest("too many tags".to_string()));
        }
        if tags.iter().any(|tag| tag.trim().is_empty()) {
            return Err(AppError::BadRequest("tags cannot be empty".to_string()));
        }
    }
    Ok(())
}

async fn ensure_tags_exist(
    state: &AppState,
    user_id: u64,
    tags: Option<&[String]>,
) -> Result<(), AppError> {
    let Some(tags) = tags else {
        return Ok(());
    };
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if bill_tag_repository::find_by_name(state.db()?, user_id, trimmed)
            .await?
            .is_none()
        {
            let _ = bill_tag_repository::create(
                state.db()?,
                &crate::dto::bill_tag::CreateBillTagRequest {
                    user_id,
                    name: trimmed.to_string(),
                    color: None,
                },
            )
            .await?;
        }
    }
    Ok(())
}

fn is_investment_category(category: &ConfigItem) -> bool {
    category.config_type == "account_category"
        && (category.name == "stock" || category.name == "wealth")
}

fn is_transfer_category(category: &ConfigItem) -> bool {
    category.config_type == "account_category" && category.name == "transfer"
}

fn derive_special_status<T>(category: &ConfigItem, payload: &T) -> String
where
    T: BillPayload,
{
    if is_transfer_category(category) {
        "transfer_pending".to_string()
    } else if is_investment_category(category) {
        "investment_pending".to_string()
    } else if payload.payment_method() == "credit_card" && payload.is_installment() {
        "debt_pending".to_string()
    } else if payload.is_fixed_asset() {
        "asset_pending".to_string()
    } else {
        "none".to_string()
    }
}

async fn handle_bill_side_effects(
    state: &AppState,
    bill_id: u64,
    category: &ConfigItem,
    payload: &CreateBillRequest,
) -> Result<(), AppError> {
    if payload.payment_method == "presale" {
        let presale_category = find_default_category(state, "presale_category", "default").await?;
        let create = CreatePresaleRequest {
            user_id: payload.user_id,
            deposit_date: payload.account_date,
            final_payment_date: None,
            category_id: presale_category.id,
            deposit_amount: payload.amount.clone(),
            final_payment_amount: Some("0.00".to_string()),
            remark: payload.remark.clone(),
        };
        let _ = presale_repository::create(
            state.db()?,
            &create,
            &presale_category.display_name,
            Some(bill_id),
        )
        .await?;
    }

    if payload.payment_method == "credit_card" && payload.is_installment.unwrap_or(false) {
        let debt_category = find_default_category(state, "debt_category", "installment").await?;
        let create = CreateDebtRequest {
            user_id: payload.user_id,
            start_date: payload.account_date,
            end_date: None,
            repay_deadline: None,
            category_id: debt_category.id,
            amount: payload.amount.clone(),
            period_count: payload.installment_months.unwrap_or(1),
            period_unit: "month".to_string(),
            period_value: 1,
            payment_method: "credit_card".to_string(),
            remark: payload.remark.clone(),
        };
        let _ = debt_repository::create(
            state.db()?,
            &create,
            &debt_category.display_name,
            Some(bill_id),
        )
        .await?;
    }

    if payload.is_fixed_asset {
        let asset_category =
            find_default_category(state, "asset_category", "default_asset").await?;
        let create = CreateAssetRequest {
            user_id: payload.user_id,
            name: payload
                .product_name
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| category.display_name.clone()),
            category_id: asset_category.id,
            amount: payload.amount.clone(),
            remark: payload.remark.clone(),
        };
        let _ = asset_repository::create(
            state.db()?,
            &create,
            &asset_category.display_name,
            Some(bill_id),
        )
        .await?;
    }

    if is_investment_category(category) {
        create_or_update_investment_from_bill(state, bill_id, category, payload).await?;
    }

    Ok(())
}

async fn find_default_category(
    state: &AppState,
    config_type: &str,
    name: &str,
) -> Result<ConfigItem, AppError> {
    config_item_repository::find_by_type_and_name(state.db()?, config_type, name)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest(format!("missing builtin config item: {config_type}/{name}"))
        })
}

async fn create_or_update_investment_from_bill(
    state: &AppState,
    bill_id: u64,
    category: &ConfigItem,
    payload: &CreateBillRequest,
) -> Result<(), AppError> {
    let pool = state.db()?;
    let investment_type = if category.name == "stock" {
        "stock"
    } else {
        "wealth"
    };
    let code = payload.product_code.as_deref().ok_or_else(|| {
        AppError::BadRequest("product_code is required for investment bill".to_string())
    })?;
    let name = payload.product_name.as_deref().ok_or_else(|| {
        AppError::BadRequest("product_name is required for investment bill".to_string())
    })?;
    let organization_name = payload.organization_name.as_deref().ok_or_else(|| {
        AppError::BadRequest("organization_name is required for investment bill".to_string())
    })?;
    let shares =
        Decimal::from_str(payload.share_amount.as_deref().unwrap_or_default()).map_err(|_| {
            AppError::BadRequest("share_amount must be a valid decimal string".to_string())
        })?;
    let amount = Decimal::from_str(&payload.amount)
        .map_err(|_| AppError::BadRequest("amount must be a valid decimal string".to_string()))?;
    let unit_price = if shares > Decimal::ZERO {
        amount / shares
    } else {
        Decimal::ZERO
    };

    let existing =
        investment_repository::find_by_user_type_code(pool, payload.user_id, investment_type, code)
            .await?;

    let (investment_id, avg_cost, realized_profit_for_tx) = if let Some(investment) = existing {
        let current_shares = Decimal::from_str(&investment.total_shares)
            .map_err(|_| AppError::internal_with_log("invalid stored total_shares"))?;
        let current_cost = Decimal::from_str(&investment.total_cost)
            .map_err(|_| AppError::internal_with_log("invalid stored total_cost"))?;
        let current_realized = Decimal::from_str(&investment.realized_profit)
            .map_err(|_| AppError::internal_with_log("invalid stored realized_profit"))?;
        let current_avg = Decimal::from_str(&investment.average_cost)
            .map_err(|_| AppError::internal_with_log("invalid stored average_cost"))?;

        let (new_shares, new_cost, new_realized, realized_profit_for_tx) =
            match payload.investment_action.as_deref().unwrap_or_default() {
                "open_position" | "add_position" => (
                    current_shares + shares,
                    current_cost + amount,
                    current_realized,
                    Decimal::ZERO,
                ),
                "reduce_position" => {
                    if shares > current_shares {
                        return Err(AppError::BadRequest(
                            "reduce_position shares exceed current holding".to_string(),
                        ));
                    }
                    let cost_reduced = current_avg * shares;
                    let tx_realized = amount - cost_reduced;
                    (
                        current_shares - shares,
                        current_cost - cost_reduced,
                        current_realized + tx_realized,
                        tx_realized,
                    )
                }
                "dividend" => (
                    current_shares,
                    current_cost,
                    current_realized + amount,
                    amount,
                ),
                _ => {
                    return Err(AppError::BadRequest(
                        "unsupported investment_action".to_string(),
                    ));
                }
            };

        let normalized_cost = if new_shares <= Decimal::ZERO {
            Decimal::ZERO
        } else {
            new_cost
        };
        let avg_cost = if new_shares > Decimal::ZERO {
            normalized_cost / new_shares
        } else {
            Decimal::ZERO
        };
        let current_price = unit_price;
        let market_value = new_shares * current_price;
        let unrealized_profit = market_value - normalized_cost;
        let total_profit = new_realized + unrealized_profit;
        let total_profit_rate = if normalized_cost > Decimal::ZERO {
            total_profit / normalized_cost
        } else {
            Decimal::ZERO
        };
        let status = if new_shares > Decimal::ZERO {
            "holding"
        } else {
            "sold"
        };

        investment_repository::update_metrics(
            pool,
            investment.id,
            name,
            organization_name,
            &format_decimal6(new_shares),
            &format_decimal2(normalized_cost),
            &format_decimal6(avg_cost),
            &format_decimal6(current_price),
            &format_decimal2(market_value),
            &format_decimal2(new_realized),
            &format_decimal2(unrealized_profit),
            &format_decimal2(total_profit),
            &format_decimal6(total_profit_rate),
            status,
        )
        .await?;

        (investment.id, avg_cost, realized_profit_for_tx)
    } else {
        if payload.investment_action.as_deref() != Some("open_position") {
            return Err(AppError::BadRequest(
                "new investment must start with open_position".to_string(),
            ));
        }
        let avg_cost = if shares > Decimal::ZERO {
            amount / shares
        } else {
            Decimal::ZERO
        };
        let current_price = unit_price;
        let market_value = shares * current_price;
        let id = investment_repository::create(
            pool,
            payload.user_id,
            bill_id,
            investment_type,
            name,
            code,
            organization_name,
            &format_decimal6(shares),
            &format_decimal2(amount),
            &format_decimal6(avg_cost),
            &format_decimal6(current_price),
            &format_decimal2(market_value),
            "0.00",
            "0.00",
            "0.00",
            "0.000000",
            "holding",
        )
        .await?;
        (id, avg_cost, Decimal::ZERO)
    };

    investment_transaction_repository::create(
        pool,
        investment_id,
        bill_id,
        payload.account_date,
        payload.investment_action.as_deref().unwrap_or_default(),
        &format_decimal6(shares),
        &format_decimal2(amount),
        &format_decimal6(unit_price.max(avg_cost)),
        &format_decimal2(realized_profit_for_tx),
        payload.remark.as_deref(),
    )
    .await?;

    Ok(())
}

fn format_decimal2(value: Decimal) -> String {
    format!("{:.2}", value.to_f64().unwrap_or(0.0))
}

fn format_decimal6(value: Decimal) -> String {
    format!("{:.6}", value.to_f64().unwrap_or(0.0))
}

trait BillPayload {
    fn payment_method(&self) -> &str;
    fn is_installment(&self) -> bool;
    fn is_fixed_asset(&self) -> bool;
}

impl BillPayload for CreateBillRequest {
    fn payment_method(&self) -> &str {
        &self.payment_method
    }

    fn is_installment(&self) -> bool {
        self.is_installment.unwrap_or(false)
    }

    fn is_fixed_asset(&self) -> bool {
        self.is_fixed_asset
    }
}

impl BillPayload for UpdateBillRequest {
    fn payment_method(&self) -> &str {
        &self.payment_method
    }

    fn is_installment(&self) -> bool {
        self.is_installment.unwrap_or(false)
    }

    fn is_fixed_asset(&self) -> bool {
        self.is_fixed_asset
    }
}

#[cfg(test)]
mod tests {
    use super::{BillPayload, derive_special_status, is_investment_category};
    use crate::model::config_item::ConfigItem;
    use chrono::NaiveDateTime;

    struct DummyPayload {
        payment_method: &'static str,
        installment: bool,
        fixed_asset: bool,
    }

    impl BillPayload for DummyPayload {
        fn payment_method(&self) -> &str {
            self.payment_method
        }

        fn is_installment(&self) -> bool {
            self.installment
        }

        fn is_fixed_asset(&self) -> bool {
            self.fixed_asset
        }
    }

    fn config_item(name: &str) -> ConfigItem {
        ConfigItem {
            id: 1,
            config_type: "account_category".to_string(),
            name: name.to_string(),
            display_name: name.to_string(),
            is_builtin: true,
            enabled: true,
            sort_order: 1,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }

    #[test]
    fn investment_category_is_detected() {
        assert!(is_investment_category(&config_item("stock")));
        assert!(is_investment_category(&config_item("wealth")));
        assert!(!is_investment_category(&config_item("food")));
    }

    #[test]
    fn special_status_prefers_transfer_then_investment() {
        let transfer = config_item("transfer");
        let payload = DummyPayload {
            payment_method: "cash",
            installment: false,
            fixed_asset: false,
        };
        assert_eq!(
            derive_special_status(&transfer, &payload),
            "transfer_pending"
        );
    }
}
