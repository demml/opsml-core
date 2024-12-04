use std::collections::HashMap;

pub struct CardQueryArgs {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub repository: Option<String>,
    pub version: Option<String>,
    pub max_date: Option<String>,
    pub tags: Option<HashMap<String, String>>,
    pub limit: Option<i32>,
    pub query_terms: Option<HashMap<String, String>>,
    pub sort_by_timestamp: Option<bool>,
}
