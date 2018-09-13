extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;
extern crate algorithmia;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;

use std::{env, thread, time};
use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::{Api, CanReplySendMessage, MessageKind, UpdateKind};
use algorithmia::Algorithmia;
use regex::Regex;


fn main() {
    //metro schedule api stuff
    let metro_api_url = env::var("METRO_API_URL").expect("Missing METRO_API_URL value");

    //Algorithmia stuff
    let alg_token = env::var("ALGORITHMIA_TOKEN").expect("Missing ALGORITHMIA_TOKEN value");
    let client = Algorithmia::client(alg_token);
    let alg = client.algo("nlp/SocialSentimentAnalysis/0.1.4");

    //Telegram stuff
    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN value");
    let api = Api::configure(token).build(core.handle()).expect("The API token may not be correct");


    loop { //using a primitive loop until I find a fancy way of dealing with the error returned by for_each when it occurs
        let future = api.stream().for_each(|update| {
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    println!("<{}>: {}", &message.from.first_name, data);
                    if is_next_arrival_request(data) {
                        let data_vec: Vec<&str> = data.splitn(2,' ').collect();
                        if let Ok(s) = next_arrival_request(&metro_api_url, NextArrivalRequest{
                            station: data_vec[1].to_string(),
                            direction: data_vec[0].to_string().to_lowercase(),
                        }){
                            api.spawn(message.text_reply(
                                format!("station: {}\ndirection: {}\ntime: {}", s.station, s.direction, s.time)
                            ));
                        }
                    } else {
                        if let Ok(resp) = alg.pipe(data) {
                            match serde_json::from_str::<Vec<SentimentResponse>>(&resp.to_string()) {
                                Ok(s) => api.spawn(message.text_reply(
                                    format!("compound:{}\nnegative:{}\nneutral:{}\npositive:{}", s[0].compound, s[0].negative, s[0].neutral, s[0].positive)
                                )),
                                Err(e) => println!("{}", e),
                            }
                        }
                    }
                }
            }
            Ok(())
        });
        match core.run(future) {
            Ok(f) => f,
            Err(f) => {
                println!("{}: trying again in 30s", f);
                thread::sleep(time::Duration::new(30, 0));
            }
        }
    }
}

#[derive(Deserialize)]
struct SentimentResponse {
    compound: f64,
    negative: f64,
    neutral: f64,
    positive: f64,
}

#[derive(Serialize)]
struct NextArrivalRequest {
    station: String,
    direction: String,
}

#[derive(Deserialize)]
struct NextArrivalResponse {
    station: String,
    direction: String,
    time: String,
}

fn is_next_arrival_request(text: &str) -> bool {
    lazy_static! {
        static ref NARE: Regex = Regex::new(r"(West|East)\s[a-z0-9]+\s?[a-z]*").unwrap();
    }
    NARE.is_match(text)
}

fn next_arrival_request(url: &str, req: NextArrivalRequest) -> Result<NextArrivalResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut res = client.post(url).json(&req).send()?;
    let next_arrival: NextArrivalResponse = res.json()?;
    Ok(next_arrival)
}



