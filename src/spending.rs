use regex::Regex;

fn spent_request(url: &str, req: SpentRequest) -> Result<SpentResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut res = client.post(url).json(&req).send()?;
    let response: SpentResponse = res.json()?;
    Ok(response)
}

//determine if request was for total, reset, or addition, and perform that action, return a formatted string of the results.
pub fn parse_spent_request(input: &str, urls: (&str, &str, &str)) -> String {
    match input {
        "reset" => match spent_get_request(urls.0) {
            Ok(s) => format!("total: {}\ntransactions: {:?}", s.total, s.transactions),
            Err(_) => "error calling api".to_string(),
        },
        "total" => match spent_get_request(urls.1) {
            Ok(s) => format!("total: {}\ntransactions: {:?}", s.total, s.transactions),
            Err(_) => "error calling api".to_string(),
        },
        _ => {
            match input.parse::<f64>() {
                Ok(_) => {
                    match spent_request(
                        urls.2,
                        SpentRequest {
                            amount: input.parse::<f64>().unwrap(), //should check this
                        },
                    ) {
                        Ok(s) => format!("total: {}", s.total),
                        Err(_) => "error calling api".to_string(),
                    }
                }
                Err(_) => "cannot parse that value as float".to_string(),
            }
        }
    }
}
fn spent_get_request(url: &str) -> Result<SpentTotalResponse, reqwest::Error> {
    let response: SpentTotalResponse = reqwest::get(url)?.json()?;
    Ok(response)
}

pub fn is_spent_request(text: &str) -> bool {
    lazy_static! {
        static ref NSRE: Regex =
            Regex::new(r"(spent|Spent)\s(total|reset|-?[0-9]+\.?[0-9]+)").unwrap();
    }
    NSRE.is_match(text)
}

pub fn help_spending() -> &'static str {
    "Spending Tracker:\nspent total\nspent reset\nspent 10.67"
}

#[derive(Serialize, Deserialize)]
pub struct SpentRequest {
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct SpentResponse {
    pub total: String,
}

#[derive(Deserialize, Serialize)]
pub struct SpentTotalResponse {
    pub total: String,
    pub transactions: Vec<String>,
}
