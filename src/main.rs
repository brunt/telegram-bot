use futures::StreamExt;
use teloxide::prelude::*;

use once_cell::sync::Lazy;

mod arrival;
mod config;
mod forecast;
mod spending;
use crate::forecast::{help_weather, weather_request};
use arrival::{help_schedule, is_next_arrival_request, next_arrival_request, NextArrivalRequest};
use config::Config;
use spending::{help_spending, is_spent_category_request, is_spent_request, parse_spent_request};

static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env());

#[tokio::main]
async fn main() {
    run().await; //recommended pattern
}

async fn run() {
    let bot = Bot::from_env();
    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, |msg| async move {
                match msg.update.text() {
                    None => {
                        if let Some(loc) = msg.update.location() {
                            msg.answer(weather_request(
                                &CONFIG.forecast_token,
                                loc.latitude as f64,
                                loc.longitude as f64,
                            ))
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
                                &CONFIG.metro_api_url,
                                NextArrivalRequest {
                                    station: data_vec[1].to_string().to_lowercase(),
                                    direction: data_vec[0].to_string().to_lowercase(),
                                },
                            ) {
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
                                msg.answer(parse_spent_request(
                                    split[1],
                                    Some(split[2].into()),
                                    (
                                        &CONFIG.spending_reset_url,
                                        &CONFIG.spending_total_url,
                                        &CONFIG.spending_add_url,
                                    ),
                                ))
                                .send()
                                .await
                                .unwrap();
                            } else {
                                msg.answer(parse_spent_request(
                                    split[1],
                                    None,
                                    (
                                        &CONFIG.spending_reset_url,
                                        &CONFIG.spending_total_url,
                                        &CONFIG.spending_add_url,
                                    ),
                                ))
                                .send()
                                .await
                                .unwrap();
                            }
                        }
                        _ => {}
                    },
                }
            })
        })
        .dispatch()
        .await;
}

fn helpmsg() -> &'static str {
    "Use the following for additional details:\nhelp schedule\nhelp spending\nhelp weather"
}
