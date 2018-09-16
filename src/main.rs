extern crate algorithmia;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate telegram_bot;
extern crate tokio_core;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;

use algorithmia::Algorithmia;
use futures::Stream;
use regex::Regex;
use std::{env, thread, time};
use telegram_bot::{Api, CanReplySendMessage, MessageKind, UpdateKind};
use tokio_core::reactor::Core;

fn main() {
    //metro schedule api stuff
    let metro_api_url = env::var("METRO_API_URL").expect("Missing METRO_API_URL value");

    //Algorithmia stuff
    let alg_token = env::var("ALGORITHMIA_TOKEN").expect("Missing ALGORITHMIA_TOKEN value");
    let client = Algorithmia::client(alg_token);

    //Telegram stuff
    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN value");
    let api = Api::configure(token)
        .build(core.handle())
        .expect("The API token may not be correct");

    loop {
        //using a primitive loop until I find a fancy way of dealing with the error returned by for_each when it occurs
        let future = api.stream().for_each(|update| {
            if let UpdateKind::Message(message) = update.kind {
                match message.kind {
                    MessageKind::Text{ ref data, ref entities } => {
                        println!("<{}>: {}, entities {:?}", &message.from.first_name, data, entities);
                        if is_next_arrival_request(data) {
                            let data_vec: Vec<&str> = data.splitn(2,' ').collect();
                            if let Ok(s) = next_arrival_request(&metro_api_url, NextArrivalRequest{
                                station: data_vec[1].to_string().to_lowercase(),
                                direction: data_vec[0].to_string().to_lowercase(),
                            }){
                                api.spawn(message.text_reply(
                                    format!("station: {}\ndirection: {}\ntime: {}", s.station, s.direction, s.time)
                                ));
                            }
                        } else if !entities.is_empty() { //a non-empty vec indicates a url was in the link
//                            for e in entities { the offset and length values are not public in the entity struct so for now I'll just assume the entire message is a link.
//                                let url = &message[e.offset..e.length];
//                                ...
//                            }
                            let alg = client.algo("nlp/SummarizeURL/0.1.4");
                            if let Ok(resp) = alg.pipe(data) {
                                api.spawn(message.text_reply(&resp.to_string()));
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
            Ok(())
        });
        match core.run(future) {
            Ok(f) => f,
            Err(f) => {
                println!("{}: trying again in 30s", f);
                thread::sleep(time::Duration::from_secs(30));
            }
        }
    }
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
        static ref NARE: Regex =
            Regex::new(r"(east|west|West|East)\s[a-zA-Z0-9]+\s?[a-zA-Z]*").unwrap();
    }
    NARE.is_match(text)
}

fn next_arrival_request(
    url: &str,
    req: NextArrivalRequest,
) -> Result<NextArrivalResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut res = client.post(url).json(&req).send()?;
    let next_arrival: NextArrivalResponse = res.json()?;
    Ok(next_arrival)
}
