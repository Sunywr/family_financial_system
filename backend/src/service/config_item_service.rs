use crate::{
    common::state::AppState,
    dto::config_item::{
        ConfigItemListQuery, ConfigTypeDto, CreateConfigItemRequest, UpdateConfigItemRequest,
    },
    error::app_error::AppError,
    model::config_item::ConfigItem,
    repository::config_item_repository,
};

pub const CONFIG_TYPES: &[ConfigTypeDto] = &[
    ConfigTypeDto {
        code: "account_category",
        label: "记账分类",
    },
    ConfigTypeDto {
        code: "debt_category",
        label: "债务分类",
    },
    ConfigTypeDto {
        code: "presale_category",
        label: "预售分类",
    },
    ConfigTypeDto {
        code: "investment_category",
        label: "投资分类",
    },
    ConfigTypeDto {
        code: "wealth_org",
        label: "理财所属机构",
    },
    ConfigTypeDto {
        code: "stock_org",
        label: "股票所属机构",
    },
    ConfigTypeDto {
        code: "asset_category",
        label: "固定资产分类",
    },
    ConfigTypeDto {
        code: "budget_category",
        label: "预算分类",
    },
    ConfigTypeDto {
        code: "brand_category",
        label: "品牌分类",
    },
    ConfigTypeDto {
        code: "system_setting",
        label: "系统设置",
    },
];

pub async fn list(
    state: &AppState,
    query: &ConfigItemListQuery,
) -> Result<(Vec<ConfigItem>, u64), AppError> {
    config_item_repository::list(state.db()?, query).await
}

pub async fn detail(state: &AppState, id: u64) -> Result<ConfigItem, AppError> {
    config_item_repository::find_by_id(state.db()?, id).await
}

pub async fn create(
    state: &AppState,
    payload: &CreateConfigItemRequest,
) -> Result<ConfigItem, AppError> {
    validate_create(payload)?;
    let id = config_item_repository::create(state.db()?, payload).await?;
    detail(state, id).await
}

pub async fn update(
    state: &AppState,
    id: u64,
    payload: &UpdateConfigItemRequest,
) -> Result<ConfigItem, AppError> {
    if payload.display_name.trim().is_empty() {
        return Err(AppError::BadRequest("display_name is required".to_string()));
    }
    config_item_repository::update(state.db()?, id, payload).await?;
    detail(state, id).await
}

pub async fn delete(state: &AppState, id: u64) -> Result<(), AppError> {
    config_item_repository::soft_delete(state.db()?, id).await
}

pub fn list_types() -> &'static [ConfigTypeDto] {
    CONFIG_TYPES
}

fn validate_create(payload: &CreateConfigItemRequest) -> Result<(), AppError> {
    if payload.config_type.trim().is_empty()
        || payload.name.trim().is_empty()
        || payload.display_name.trim().is_empty()
    {
        return Err(AppError::BadRequest(
            "config_type, name and display_name are required".to_string(),
        ));
    }
    if !CONFIG_TYPES
        .iter()
        .any(|item| item.code == payload.config_type)
    {
        return Err(AppError::BadRequest("unsupported config_type".to_string()));
    }
    Ok(())
}
