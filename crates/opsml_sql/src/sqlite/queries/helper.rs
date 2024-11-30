const GET_CARDS_WITH_VERSION: &str = include_str!("scripts/get_cards_with_version.sql");
const GET_CARDS_WITHOUT_VERSION: &str = include_str!("scripts/get_cards_without_version.sql");

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
    GetCardsWithVersion,
    GetCardsWithoutVersion,
}

impl Queries {
    pub fn get_query(&self) -> SqlQuery {
        match self {
            Queries::GetCardsWithVersion => SqlQuery::new(GET_CARDS_WITH_VERSION),
            Queries::GetCardsWithoutVersion => SqlQuery::new(GET_CARDS_WITHOUT_VERSION),
        }
    }
}
