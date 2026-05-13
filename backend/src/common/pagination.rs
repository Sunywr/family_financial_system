use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page", deserialize_with = "deserialize_u64")]
    pub page: u64,
    #[serde(default = "default_page_size", deserialize_with = "deserialize_u64")]
    pub page_size: u64,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

impl PaginationQuery {
    pub fn offset(&self) -> u64 {
        self.page.saturating_sub(1) * self.page_size
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageData<T> {
    pub list: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

pub fn default_page() -> u64 {
    1
}

pub fn default_page_size() -> u64 {
    20
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse::<u64>().map_err(serde::de::Error::custom)
}
