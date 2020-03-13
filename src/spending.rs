use lazy_static::*;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

fn spent_request(url: &str, req: SpentRequest) -> Result<SpentResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut res = client.post(url).json(&req).send()?;
    let response: SpentResponse = res.json()?;
    Ok(response)
}

//determine if request was for total, reset, or addition, and perform that action, return a formatted string of the results.
pub fn parse_spent_request(
    input: &str,
    category: Option<Category>,
    urls: (&str, &str, &str),
) -> String {
    match input {
        "reset" => match spent_get_request(urls.0) {
            Ok(s) => s.to_string(),
            Err(_) => "error calling api".to_string(),
        },
        "total" => match spent_get_request(urls.1) {
            Ok(s) => s.to_string(),
            Err(_) => "error calling api".to_string(),
        },
        _ => match input.parse::<f64>() {
            Ok(amount) => match spent_request(urls.2, SpentRequest { amount, category }) {
                Ok(s) => s.to_string(),
                Err(_) => "error calling api".to_string(),
            },
            Err(_) => "cannot parse that value as float".to_string(),
        },
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

pub fn is_spent_category_request(text: &str) -> bool {
    lazy_static! {
        static ref NSREC: Regex =
            Regex::new(r"(spent|Spent)\s(total|reset|-?[0-9]+\.?[0-9]+)\s(dining|travel|merchandise|entertainment|other)").unwrap();
    }
    NSREC.is_match(text)
}

pub fn help_spending() -> &'static str {
    "Spending Tracker:\nspent total\nspent reset\nspent 10.67"
}

#[derive(Serialize, Deserialize)]
pub struct SpentRequest {
    pub amount: f64,
    pub category: Option<Category>,
}

#[derive(Deserialize)]
pub struct SpentResponse {
    pub total: String,
}

impl fmt::Display for SpentResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "total: {}", self.total)
    }
}

#[derive(Deserialize, Serialize)]
pub struct SpentTotalResponse {
    pub total: String,
    pub transactions: Vec<(String, Category)>,
}

impl fmt::Display for SpentTotalResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "total: {}\ntransactions: {:?}",
            self.total, self.transactions
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Category {
    Dining,
    Travel,
    Merchandise,
    Entertainment,
    Other,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print = match *self {
            Category::Dining => "Dining",
            Category::Travel => "Travel",
            Category::Merchandise => "Merchandise",
            Category::Entertainment => "Entertainment",
            Category::Other => "Other",
        };
        write!(f, "{}", print)
    }
}

impl std::convert::From<&str> for Category {
    fn from(s: &str) -> Self {
        match s {
            "Dining" | "dining" => Category::Dining,
            "Travel" | "travel" => Category::Travel,
            "Merchandise" | "merchandise" => Category::Merchandise,
            "Entertainment" | "entertainment" => Category::Entertainment,
            "Other" | "other" => Category::Other,
            _ => Category::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_spent_request() {
        assert_eq!(is_spent_request("spent total"), true);
        assert_eq!(is_spent_request("spent reset"), true);
        assert_eq!(is_spent_request("spent 0.01"), true);
        assert_eq!(is_spent_request("spent 1000"), true);
        assert_eq!(is_spent_request("spent -4"), false);
    }

    #[test]
    fn test_is_spent_category_request() {
        assert_eq!(is_spent_category_request("spent 10.00 dining"), true);
        assert_eq!(is_spent_category_request("spent 10.00 entertainment"), true);
        assert_eq!(is_spent_category_request("spent 10.00 merchandise"), true);
        assert_eq!(is_spent_category_request("spent 10.00 travel"), true);
        assert_eq!(is_spent_category_request("spent 10.00 other"), true);
        assert_eq!(is_spent_category_request("spent 10.00 something"), false);
    }
}
