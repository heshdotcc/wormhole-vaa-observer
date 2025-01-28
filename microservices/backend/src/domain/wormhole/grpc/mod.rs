pub mod proto;
pub mod client;
pub mod handlers;
pub mod vaa;

// Re-export scoped routes
pub use handlers::wormhole_routes as spy_routes;

// Re-export proto types
pub use proto::spy::v1::{
    SubscribeSignedVaaRequest,
    SubscribeSignedVaaResponse,
    EmitterFilter,
};
pub use proto::gossip::v1::Heartbeat;
pub use proto::publicrpc::v1::ChainId; 