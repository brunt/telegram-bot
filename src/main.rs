use actix_web::{App, HttpServer};
use actix_web_prom::{PrometheusMetrics};
use futures::StreamExt;
use prometheus::{opts, IntCounterVec};
use std::sync::Arc;
use teloxide::prelude::*;


mod arrival;
mod config;
mod spending;
mod weather;
use arrival::{help_schedule, is_next_arrival_request, next_arrival_request, NextArrivalRequest};
use config::Config;
use spending::{help_spending, is_spent_category_request, is_spent_request, parse_spent_request};
use weather::{help_weather, weather_request};

#[actix_rt::main]
async fn main() {
    let prometheus = PrometheusMetrics::new("teloxide", Some("/metrics"), None);
    let counter_opts = opts!("counter", "requests").namespace("teloxide");
    let counter = IntCounterVec::new(counter_opts, &["request"]).unwrap();
    prometheus
        .registry
        .register(Box::new(counter.clone())).unwrap();
    let config = Config::from_env();
    HttpServer::new(move || App::new().wrap(prometheus.clone()))
        .bind(format!("0.0.0.0:{}", &config.webserver_port))
        .expect("address in use")
        .run();
    run(config, counter).await;
}

async fn run(config: Arc<Config>, counter: IntCounterVec) {
    let bot = Bot::from_env();
    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, move |msg| {
                let config = Arc::clone(&config);
                let counter = counter.clone();
                async move {
                    match msg.update.text() {
                        None => {
                            if let Some(loc) = msg.update.location() {
                                counter.with_label_values(&["Weather"]).inc();
                                msg.answer(
                                    weather_request(
                                        &config.forecast_token,
                                        loc.latitude as f64,
                                        loc.longitude as f64,
                                    )
                                    .await,
                                )
                                .send()
                                .await
                                .unwrap();
                            }
                        }
                        Some(txt) => match txt {
                            x if x.eq("Help") => {
                                counter.with_label_values(&["Help"]).inc();
                                msg.answer(helpmsg()).send().await.unwrap();
                            }
                            x if x.eq("Help schedule") => {
                                counter.with_label_values(&["Help schedule"]).inc();
                                msg.answer(help_schedule()).send().await.unwrap();
                            }
                            x if x.eq("Help spending") => {
                                counter.with_label_values(&["Help spending"]).inc();
                                msg.answer(help_spending()).send().await.unwrap();
                            }
                            x if x.eq("Help weather") => {
                                counter.with_label_values(&["Help weather"]).inc();
                                msg.answer(help_weather()).send().await.unwrap();
                            }
                            x if is_next_arrival_request(x) => {
                                counter.with_label_values(&["Next Arrival"]).inc();
                                let data_vec: Vec<&str> = x.splitn(2, ' ').collect();
                                match next_arrival_request(
                                    &config.metro_api_url,
                                    NextArrivalRequest {
                                        station: data_vec[1].to_string().to_lowercase(),
                                        direction: data_vec[0].to_string().to_lowercase(),
                                    },
                                )
                                .await
                                {
                                    Ok(s) => {
                                        msg.answer(s.to_string()).send().await.unwrap();
                                    }
                                    Err(_) => {
                                        msg.answer("An error occurred retrieving the schedule")
                                            .send()
                                            .await
                                            .unwrap();
                                    }
                                }
                            }
                            x if is_spent_request(x) => {
                                counter.with_label_values(&["Spending"]).inc();
                                let split: Vec<&str> = x.split(' ').collect();
                                if is_spent_category_request(x) {
                                    msg.answer(
                                        parse_spent_request(
                                            split[1],
                                            Some(split[2].into()),
                                            (
                                                &config.spending_reset_url,
                                                &config.spending_total_url,
                                                &config.spending_add_url,
                                            ),
                                        )
                                        .await,
                                    )
                                    .send()
                                    .await
                                    .unwrap();
                                } else {
                                    msg.answer(
                                        parse_spent_request(
                                            split[1],
                                            None,
                                            (
                                                &config.spending_reset_url,
                                                &config.spending_total_url,
                                                &config.spending_add_url,
                                            ),
                                        )
                                        .await,
                                    )
                                    .send()
                                    .await
                                    .unwrap();
                                }
                            }
                            _ => {}
                        },
                    }
                }
            })
        })
        .dispatch()
        .await;
}

fn helpmsg() -> &'static str {
    "Use the following for additional details:\nhelp schedule\nhelp spending\nhelp weather"
}
