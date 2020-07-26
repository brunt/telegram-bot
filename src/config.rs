use crate::metro::MetroScheduleAPI;
use crate::spending::SpendingAPI;
use std::env;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) spending_api: SpendingAPI,
    pub(crate) metro_api: MetroScheduleAPI,
    pub(crate) forecast_token: String,
    pub(crate) webserver_port: String,
}

impl Config {
    pub(crate) fn from_env() -> Arc<Config> {
        Arc::new(Config {
            spending_api: SpendingAPI {
                spending_add_url: env::var("SPENDING_API_ADD")
                    .expect("Missing SPENDING_API_URL value"),
                spending_total_url: env::var("SPENDING_API_TOTAL")
                    .expect("Missing SPENDING_API_URL value"),
                spending_reset_url: env::var("SPENDING_API_RESET")
                    .expect("Missing SPENDING_API_URL value"),
                budget_set_url: env::var("SPENDING_API_BUDGET_URL")
                    .expect("Missing SPENDING_API_BUDGET_URL"),
            },
            metro_api: MetroScheduleAPI {
                url: env::var("METRO_API_URL").expect("Missing METRO_API_URL value"),
            },
            forecast_token: env::var("FORECAST_TOKEN").expect("Missing FORECAST_TOKEN"),
            webserver_port: env::var("BOT_METRICS_PORT").expect("Missing BOT_METRICS_PORT value"),
        })
    }
}
