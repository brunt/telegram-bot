extern crate algorithmia;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate telegram_bot_fork;
extern crate tokio;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;

use algorithmia::Algorithmia;
use futures::{future::lazy, Stream};
use std::{env, thread, time};
use telegram_bot_fork::{Api, CanReplySendMessage, Error, MessageKind, Update, UpdateKind};

mod arrival;
mod spending;
use arrival::{is_next_arrival_request, next_arrival_request, NextArrivalRequest};
use spending::{is_spent_request, parse_spent_request};

fn main() {
    tokio::runtime::current_thread::Runtime::new()
        .expect("error creating new runtime")
        .block_on(lazy(|| {
            //metro schedule api stuff
            let metro_api_url = env::var("METRO_API_URL").expect("Missing METRO_API_URL value");
            //spending tracker api
            let spending_add_url =
                env::var("SPENDING_API_ADD").expect("Missing SPENDING_API_URL value");
            let spending_total_url =
                env::var("SPENDING_API_TOTAL").expect("Missing SPENDING_API_URL value");
            let spending_reset_url =
                env::var("SPENDING_API_RESET").expect("Missing SPENDING_API_URL value");

            //Algorithmia stuff
            let alg_token = env::var("ALGORITHMIA_TOKEN").expect("Missing ALGORITHMIA_TOKEN value");
            let client = Algorithmia::client(alg_token);

            let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
            let api = Api::new(token).unwrap();

            // Convert stream to the stream with errors in result
            let stream = api.stream().then(|mb_update| {
                let res: Result<Result<Update, Error>, ()> = Ok(mb_update);
                res
            });

            // Print update or error for each update.
            stream.for_each(move |update| {
                match update {
                    Ok(update) => {
                        if let UpdateKind::Message(message) = update.kind {
                            match message.kind {
                                MessageKind::Text{ ref data, ref entities } => {
                                    println!("<{}>: {}, entities {:?}", &message.from.first_name, data, entities);
                                    match data {
                                        x if is_next_arrival_request(x) => {
                                            let data_vec: Vec<&str> = x.splitn(2, ' ').collect();
                                            if let Ok(s) = next_arrival_request(&metro_api_url, NextArrivalRequest {
                                                station: data_vec[1].to_string().to_lowercase(),
                                                direction: data_vec[0].to_string().to_lowercase(),
                                            }) {
                                                api.spawn(message.text_reply(format!("station: {}\ndirection: {}\nline: {}\ntime: {}", s.station, s.direction, s.line, s.time)
                                                ));
                                            }
                                        },
                                        x if is_spent_request(x) => {
                                            let split: Vec<&str> = x.split(' ').collect();
                                            api.spawn(message.text_reply(parse_spent_request(split[1], (&spending_reset_url, &spending_total_url, &spending_add_url))));
                                        },
                                        _ => {
                                            if !entities.is_empty() { //a non-empty vec indicates a url was in the link
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
                                },
                                MessageKind::Photo { ref data, .. } => {
                                    println!("{}: {:?}", &message.from.first_name, data);
                                    api.spawn(message.text_reply("That's a great pic and I'm still trying to figure out how to interact with it."));
                                },
                                _ => (),
                            }
                        }
                    },
                    Err(e) => {
                        println!("{}: trying again in 30s", e);
                        thread::sleep(time::Duration::from_secs(30));
                    }
                }
                Ok(())
            })
        })).expect("error running future");
}
