use std::env;
use dotenv::dotenv;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub api_title: String,
    pub wormholescan_base_url: String,
    pub wormhole_spy_addr: Option<String>,
    pub wormhole_spy_timeout: u64,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn new() -> Self {
        // Load .env file if it exists
        dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            host: env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_title: env::var("API_TITLE")
                .unwrap_or_else(|_| "Wormhole VAA Observer API".to_string()),
            wormholescan_base_url: env::var("WORMHOLESCAN_BASE_URL")
                .unwrap_or_else(|_| "https://api.wormholescan.io".to_string()),
            wormhole_spy_addr: env::var("WORMHOLE_SPY_ADDR")
                .unwrap_or_else(|_| "http://127.0.0.1:7073".to_string())
                .into(),
            wormhole_spy_timeout: env::var("WORMHOLE_SPY_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("WORMHOLE_SPY_TIMEOUT must be a number"),
        }
    }
}

// Helper function to get config singleton
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::new())
} 