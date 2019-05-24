use regex::Regex;
use std::fmt;

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

pub fn help_schedule() -> &'static str {
    "Next Arrival:\nGet the next arriving train on the STL Metro\nType East or West followed by a station name e.g. \"West fvh\"\nstation names:\n
    lambert\n
    lambert2\n
    hanley\n
    umsl north (umsl)\n
    umsl south\n
    rock road\n
    wellston\n
    delmar\n
    shrewsbury\n
    sunnen\n
    maplewood\n
    brentwood\n
    richmond\n
    clayton\n
    forsyth\n
    u city\n
    skinker\n
    forest park\n
    cwe (central west end)\n
    cortex\n
    grand\n
    union\n
    civic (civic center)\n
    stadium\n
    8th pine (8th and pine)\n
    convention (convention center\n
    lacledes (lacledes landing)\n
    riverfront (east riverfront)\n
    5th missouri (fifth missouri)\n
    emerson\n
    jjk (jackie joiner)\n
    washington\n
    fvh (fairview heights)\n
    memorial hospital\n
    swansea\n
    belleville\n
    college\n
    shiloh (shiloh scott)"
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

impl fmt::Display for NextArrivalResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "station: {}\ndirection: {}\nline: {}\ntime: {}",
                   self.station,
                   self.direction,
                   self.line,
                   self.time)
    }
}
