use algorithmia::Algorithmia;
use futures::StreamExt;
use std::env;
use telegram_bot::{Api, CanReplySendMessage, MessageKind, UpdateKind};

mod arrival;
mod forecast;
mod spending;
use crate::forecast::{help_weather, weather_request};
use arrival::{help_schedule, is_next_arrival_request, next_arrival_request, NextArrivalRequest};
use spending::{help_spending, is_spent_category_request, is_spent_request, parse_spent_request};

#[tokio::main]
async fn main() {
    let metro_api_url = env::var("METRO_API_URL").expect("Missing METRO_API_URL value");
    let spending_add_url = env::var("SPENDING_API_ADD").expect("Missing SPENDING_API_URL value");
    let spending_total_url =
        env::var("SPENDING_API_TOTAL").expect("Missing SPENDING_API_URL value");
    let spending_reset_url =
        env::var("SPENDING_API_RESET").expect("Missing SPENDING_API_URL value");
    let forecast_token = env::var("FORECAST_TOKEN").expect("Missing FORECAST_TOKEN");
    let alg_token = env::var("ALGORITHMIA_TOKEN").expect("Missing ALGORITHMIA_TOKEN value");
    let client = Algorithmia::client(alg_token);

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN value");
    let api = Api::new(token);

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        if let Ok(update) = update {
            if let UpdateKind::Message(message) = update.kind {
                match message.kind {
                    MessageKind::Text {
                        ref data,
                        ref entities,
                    } => {
                        match data {
                            x if x.eq("Help") => {
                                api.spawn(message.text_reply(helpmsg()));
                            }
                            x if x.eq("Help schedule") => {
                                api.spawn(message.text_reply(help_schedule()));
                            }
                            x if x.eq("Help spending") => {
                                api.spawn(message.text_reply(help_spending()));
                            }
                            x if x.eq("Help weather") => {
                                api.spawn(message.text_reply(help_weather()));
                            }
                            x if is_next_arrival_request(x) => {
                                let data_vec: Vec<&str> = x.splitn(2, ' ').collect();
                                match next_arrival_request(
                                    &metro_api_url,
                                    NextArrivalRequest {
                                        station: data_vec[1].to_string().to_lowercase(),
                                        direction: data_vec[0].to_string().to_lowercase(),
                                    },
                                ) {
                                    Ok(s) => {
                                        api.spawn(message.text_reply(s.to_string()));
                                    }
                                    Err(_) => {
                                        api.spawn(message.text_reply(
                                            "An error occurred retrieving the schedule",
                                        ))
                                    }
                                }
                            }
                            x if is_spent_request(x) => {
                                let split: Vec<&str> = x.split(' ').collect();
                                if is_spent_category_request(x) {
                                    api.spawn(message.text_reply(parse_spent_request(
                                        split[1],
                                        Some(split[2].into()),
                                        (
                                            &spending_reset_url,
                                            &spending_total_url,
                                            &spending_add_url,
                                        ),
                                    )));
                                } else {
                                    api.spawn(message.text_reply(parse_spent_request(
                                        split[1],
                                        None,
                                        (
                                            &spending_reset_url,
                                            &spending_total_url,
                                            &spending_add_url,
                                        ),
                                    )));
                                }
                            }
                            _ => {
                                if !entities.is_empty() {
                                    //a non-empty vec indicates a url was in the link
                                    // for e in entities { //the offset and length values are not public in the entity struct so for now I'll just assume the entire message is a link.
                                    //     let url = &message[e.offset..e.length];
                                    //     ...
                                    // }
                                    let alg = client.algo("nlp/SummarizeURL/0.1.4");
                                    if let Ok(resp) = alg.pipe(data) {
                                        api.spawn(message.text_reply(&resp.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    MessageKind::Location { ref data } => {
                        api.spawn(message.text_reply(weather_request(
                            &forecast_token,
                            data.latitude as f64,
                            data.longitude as f64,
                        )))
                    }
                    _ => {} //MessageKind::Picture, etc
                }
            }
        }
    }
}

fn helpmsg() -> &'static str {
    "Use the following for additional details:\nhelp schedule\nhelp spending\nhelp summarize\nhelp weather"
}
