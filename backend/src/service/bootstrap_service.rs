use crate::{
    common::state::AppState,
    error::app_error::AppError,
    repository::{config_item_repository, user_repository},
};

const BUILTIN_CONFIG_ITEMS: &[(&str, &str, &str, i32)] = &[
    ("account_category", "stock", "股票", 1),
    ("account_category", "wealth", "理财", 2),
    ("account_category", "food", "饮食", 3),
    ("account_category", "living", "生活", 4),
    ("account_category", "transport", "交通", 5),
    ("account_category", "snack", "零食", 6),
    ("account_category", "housing", "住房", 7),
    ("account_category", "transfer", "转账", 8),
    ("debt_category", "credit_card", "信用卡", 1),
    ("debt_category", "loan", "贷款", 2),
    ("debt_category", "installment", "分期", 3),
    ("presale_category", "default", "通用预售", 1),
    ("investment_category", "wealth", "理财", 1),
    ("investment_category", "stock", "股票", 2),
    ("wealth_org", "default_wealth_org", "默认理财机构", 1),
    ("stock_org", "default_stock_org", "默认券商", 1),
    ("asset_category", "default_asset", "通用固定资产", 1),
    ("budget_category", "default_budget", "通用预算分类", 1),
    ("brand_category", "default_brand", "通用品牌分类", 1),
    (
        "system_setting",
        "use_provident_fund_for_mortgage",
        "公积金冲抵房贷",
        1,
    ),
];

pub async fn seed_builtin_items(state: &AppState) -> Result<(), AppError> {
    let pool = state.db()?;
    user_repository::seed_default_admin(pool).await?;
    config_item_repository::seed_builtin_items(pool, BUILTIN_CONFIG_ITEMS).await
}
