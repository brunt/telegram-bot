use regex::Regex;

pub fn is_next_arrival_request(text: &str) -> bool {
    lazy_static! {
        static ref NARE: Regex =
            Regex::new(r"(east|west|West|East)\s[a-zA-Z0-9]+\s?[a-zA-Z]*").unwrap();
    }
    NARE.is_match(text)
}

pub fn next_arrival_request(
    url: &str,
    req: NextArrivalRequest,
) -> Result<NextArrivalResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut res = client.post(url).json(&req).send()?;
    let next_arrival: NextArrivalResponse = res.json()?;
    Ok(next_arrival)
}

#[derive(Serialize)]
pub struct NextArrivalRequest {
    pub station: String,
    pub direction: String,
}

#[derive(Deserialize)]
pub struct NextArrivalResponse {
    pub station: String,
    pub direction: String,
    pub line: String,
    pub time: String,
}
