use std::env;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub metro_api_url: String,
    pub spending_add_url: String,
    pub spending_total_url: String,
    pub spending_reset_url: String,
    pub spending_budget_url: String,
    pub forecast_token: String,
    pub webserver_port: String,
}

impl Config {
    pub(crate) fn from_env() -> Arc<Config> {
        Arc::new(Config {
            metro_api_url: env::var("METRO_API_URL").expect("Missing METRO_API_URL value"),
            spending_add_url: env::var("SPENDING_API_ADD").expect("Missing SPENDING_API_URL value"),
            spending_total_url: env::var("SPENDING_API_TOTAL")
                .expect("Missing SPENDING_API_URL value"),
            spending_reset_url: env::var("SPENDING_API_RESET")
                .expect("Missing SPENDING_API_URL value"),
            spending_budget_url: env::var("SPENDING_API_BUDGET_URL")
                .expect("Missing SPENDING_API_BUDGET_URL"),
            forecast_token: env::var("FORECAST_TOKEN").expect("Missing FORECAST_TOKEN"),
            webserver_port: env::var("BOT_METRICS_PORT").expect("Missing BOT_METRICS_PORT value"),
        })
    }
}
