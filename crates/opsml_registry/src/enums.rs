pub enum OpsmlRegistry {
    ClientRegistry(crate::client::registry::ClientRegistry),

    #[cfg(feature = "server")]
    ServerRegistry(crate::server::registry::server_logic::ServerRegistry),
}
