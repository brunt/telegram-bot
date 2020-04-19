use futures::StreamExt;
use teloxide::prelude::*;

use std::sync::Arc;

mod arrival;
mod config;
mod spending;
mod weather;
use arrival::{help_schedule, is_next_arrival_request, next_arrival_request, NextArrivalRequest};
use config::Config;
use spending::{help_spending, is_spent_category_request, is_spent_request, parse_spent_request};
use weather::{help_weather, weather_request};

#[tokio::main]
async fn main() {
    run().await; //recommended pattern
}

async fn run() {
    let bot = Bot::from_env();
    let config = Config::from_env();
    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, move |msg| {
                let config = Arc::clone(&config);
                async move {
                    match msg.update.text() {
                        None => {
                            if let Some(loc) = msg.update.location() {
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
                                msg.answer(helpmsg()).send().await.unwrap();
                            }
                            x if x.eq("Help schedule") => {
                                msg.answer(help_schedule()).send().await.unwrap();
                            }
                            x if x.eq("Help spending") => {
                                msg.answer(help_spending()).send().await.unwrap();
                            }
                            x if x.eq("Help weather") => {
                                msg.answer(help_weather()).send().await.unwrap();
                            }
                            x if is_next_arrival_request(x) => {
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
