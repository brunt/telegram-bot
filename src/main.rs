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
    let alg_token = env::var("ALGORITHMIA_TOKEN").expect("Missing ALGORITHMIA_TOKEN value");
    let client = Algorithmia::client(alg_token);
    let alg = client.algo("nlp/SocialSentimentAnalysis/0.1.4");

    //Telegram stuff
    let mut core = Core::new().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN value");
    let api = Api::configure(token).build(core.handle()).expect("The API token may not be correct");

    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {
        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text {ref data, ..} = message.kind {
                println!("<{}>: {}", &message.from.first_name, data);
                if let Ok(resp) = alg.pipe(data) {
                    match serde_json::from_str::<Vec<SentimentResponse>>(&resp.to_string()){
                        Ok(s) => api.spawn(message.text_reply(
                            format!("compound:{}\nnegative:{}\nneutral:{}\npositive:{}", s[0].compound, s[0].negative, s[0].neutral, s[0].positive)
                        )),
                        Err(e) => println!("{}", e),
                    }
                }

            }
        }
        Ok(())
    });
    match core.run(future) {
        Ok(f) => f,
        Err(f) => println!("{}", f),
    }
}

#[derive(Deserialize)]
struct SentimentResponse {
    compound: f64,
    negative: f64,
    neutral: f64,
    positive: f64,
}