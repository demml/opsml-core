use crate::core::health::schema::Alive;

pub async fn health_check() -> Alive {
    Alive::default()
}
