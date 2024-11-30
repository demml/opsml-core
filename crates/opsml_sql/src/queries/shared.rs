const GET_VERSIONS: &str = include_str!("scripts/get_version.sql");

pub struct SqlQuery {
    pub sql: String,
}

impl SqlQuery {
    fn new(sql: &str) -> Self {
        Self {
            sql: sql.to_string(),
        }
    }
}

#[allow(dead_code)]
pub enum Queries {
    GetVersions,
}

impl Queries {
    pub fn get_query(&self) -> SqlQuery {
        match self {
            Queries::GetVersions => SqlQuery::new(GET_VERSIONS),
        }
    }
}
