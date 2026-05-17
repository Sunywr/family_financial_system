use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InstitutionCatalogQuery {
    pub investment_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstitutionProvider {
    pub name: String,
    pub source_module: String,
    pub source_method: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstitutionCategoryDto {
    pub code: String,
    pub name: String,
    pub investment_type: String,
    pub providers: Vec<InstitutionProvider>,
}
