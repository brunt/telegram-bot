use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetrics;
use prometheus::{opts, IntCounterVec};
use std::sync::Arc;
use teloxide::prelude::*;

mod metro;
mod config;
mod dispatch;
mod spending;
mod weather;

use crate::dispatch::parse_messages;
use config::Config;

#[actix_rt::main]
async fn main() {
    let prometheus = PrometheusMetrics::new("teloxide", Some("/metrics"), None);
    let counter_opts = opts!("counter", "requests").namespace("teloxide");
    let counter = IntCounterVec::new(counter_opts, &["request"]).unwrap();
    prometheus
        .registry
        .register(Box::new(counter.clone()))
        .unwrap();
    let config = Config::from_env();
    run_webserver(&config, prometheus);
    run_chatbot(config, counter).await;
}

fn run_webserver(config: &Arc<Config>, prometheus: PrometheusMetrics) {
    HttpServer::new(move || App::new().wrap(prometheus.clone()))
        .bind(format!("0.0.0.0:{}", &config.webserver_port))
        .expect("address in use")
        .run();
}

async fn run_chatbot(config: Arc<Config>, counter: IntCounterVec) {
    let bot = Bot::from_env();
    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, move |msg| {
                let config = Arc::clone(&config);
                let counter = counter.clone();
                parse_messages(msg, config, counter)
            })
        })
        .dispatch()
        .await;
}
