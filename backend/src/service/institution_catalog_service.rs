use crate::{
    dto::institution::{InstitutionCategoryDto, InstitutionProvider},
    error::app_error::AppError,
};

pub async fn list(investment_type: Option<&str>) -> Result<Vec<InstitutionCategoryDto>, AppError> {
    let categories = vec![
        InstitutionCategoryDto {
            code: "wealth_bank".to_string(),
            name: "银行理财".to_string(),
            investment_type: "wealth".to_string(),
            providers: vec![
                provider("兴银理财", "account.api.cibwm", "get_cibwm_finance_info_by_code"),
                provider(
                    "上银理财",
                    "account.api.bankofshanghai",
                    "get_bank_of_shanghai_finance_info_by_code",
                ),
                provider("建行理财", "account.api.ccb", "get_ccb_finance_info_by_code"),
                provider(
                    "交银理财",
                    "account.api.bocommwm",
                    "get_bocommwm_finance_info_by_code",
                ),
                provider("招银理财", "account.api.cmb", "get_cmb_finance_info_by_code"),
                provider("光大理财", "account.api.cebwm", "get_cebwm_finance_info_by_code"),
                provider("信银理财", "account.api.citic", "get_citic_funds_info_by_code"),
                provider("中银理财", "account.api.bocwm", "get_bocwm_finance_info_by_code"),
                provider("农银理财", "account.api.abc", "get_abc_finance_info_by_code"),
            ],
        },
        InstitutionCategoryDto {
            code: "wealth_fund".to_string(),
            name: "基金公司理财".to_string(),
            investment_type: "wealth".to_string(),
            providers: vec![
                provider("华安基金", "account.api.huaan", "get_huaan_funds_info_by_code"),
                provider("南方基金", "account.api.nffund", "get_nffund_finance_info_by_code"),
                provider("易方达基金", "account.api.efunds", "get_efunds_finance_info_by_code"),
                provider("东方财富", "account.api.eastmoney", "get_stock_or_etf_info_by_code"),
                provider(
                    "华泰柏瑞",
                    "account.api.huataipb",
                    "get_huataipb_finance_info_by_code",
                ),
            ],
        },
        InstitutionCategoryDto {
            code: "stock_quote".to_string(),
            name: "股票行情源".to_string(),
            investment_type: "stock".to_string(),
            providers: vec![
                provider("上海证券", "account.api.sse", "get_stock_info_by_code"),
                provider("深圳证券", "account.api.szse", "get_sz_stock_info_by_code"),
            ],
        },
    ];

    if let Some(value) = investment_type {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Ok(categories);
        }
        if normalized != "stock" && normalized != "wealth" {
            return Err(AppError::BadRequest(
                "investment_type must be stock or wealth".to_string(),
            ));
        }
        return Ok(
            categories
                .into_iter()
                .filter(|item| item.investment_type == normalized)
                .collect(),
        );
    }

    Ok(categories)
}

fn provider(name: &str, source_module: &str, source_method: &str) -> InstitutionProvider {
    InstitutionProvider {
        name: name.to_string(),
        source_module: source_module.to_string(),
        source_method: source_method.to_string(),
    }
}
