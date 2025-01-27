pub mod models;
pub mod rest;
pub use rest::scan_routes;
pub use rest::client::RestClient;
pub use models::{VaaRequest, VaaResponse};