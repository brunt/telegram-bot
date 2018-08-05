extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;
extern crate algorithmia;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::env;
use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;
use algorithmia::Algorithmia;

fn main() {
    //Algorithmia stuff
    let alg_token = env::var("ALGORITHMIA_TOKEN").unwrap();
    let client = Algorithmia::client(alg_token);
    let alg = client.algo("nlp/SocialSentimentAnalysis/0.1.4");

    //Telegram stuff
    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {
        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text {ref data, ..} = message.kind {
                println!("<{}>: {}", &message.from.first_name, data);
                let resp = alg.pipe(data).unwrap();
                let s: Vec<SentimentResponse> = serde_json::from_str(&resp.to_string()).unwrap();
                api.spawn(message.text_reply(
                    format!("compound:{}\nnegative:{}\nneutral:{}\npositive:{}", s[0].compound, s[0].negative, s[0].neutral, s[0].positive)
                ));
            }
        }
        Ok(())
    });

    core.run(future).unwrap();
}

#[derive(Deserialize)]
struct SentimentResponse {
    compound: f64,
    negative: f64,
    neutral: f64,
    positive: f64,
}