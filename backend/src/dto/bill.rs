use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::common::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
pub struct BillListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub category_id: Option<u64>,
    pub bill_type: Option<String>,
    pub payment_method: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBillRequest {
    pub user_id: u64,
    pub account_date: NaiveDate,
    pub category_id: u64,
    pub bill_type: String,
    pub payment_method: String,
    pub is_fixed_asset: bool,
    pub amount: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<String>,
    pub transfer_target_type: Option<String>,
    pub transfer_target_user_id: Option<u64>,
    pub credit_card_id: Option<u64>,
    pub is_installment: Option<bool>,
    pub installment_months: Option<u32>,
    pub investment_action: Option<String>,
    pub related_investment_id: Option<u64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub organization_name: Option<String>,
    pub share_amount: Option<String>,
    pub related_asset_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBillRequest {
    pub account_date: NaiveDate,
    pub category_id: u64,
    pub bill_type: String,
    pub payment_method: String,
    pub is_fixed_asset: bool,
    pub amount: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<String>,
    pub transfer_target_type: Option<String>,
    pub transfer_target_user_id: Option<u64>,
    pub credit_card_id: Option<u64>,
    pub is_installment: Option<bool>,
    pub installment_months: Option<u32>,
    pub investment_action: Option<String>,
    pub related_investment_id: Option<u64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub organization_name: Option<String>,
    pub share_amount: Option<String>,
    pub related_asset_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct BillOptionsDto {
    pub payment_methods: &'static [&'static str],
    pub normal_bill_types: &'static [&'static str],
    pub investment_bill_types: &'static [&'static str],
    pub transfer_target_types: &'static [&'static str],
}
