use forecast::{ApiClient, ApiResponse, ExcludeBlock, ForecastRequestBuilder};
use lazy_static::*;
use regex::Regex;
use reqwest::Client;
use chrono::{Timelike, DateTime, Local, TimeZone, Utc};

pub fn help_weather() -> &'static str {
    r#"Weather examples:
    weather now
    weather today
    Powered by Dark Sky
    https://darksky.net/poweredby/"#
}

pub fn is_weather_request(text: &str) -> bool {
    //TODO: handle just 'weather'?
    lazy_static! {
        static ref WRE: Regex = Regex::new(r"(W|w)eather\s(now|today).*").unwrap();
    }
    WRE.is_match(text)
}

pub fn parse_weather_request(context: &str, token: &str) -> String {
    let req = Client::new();
    let call = ApiClient::new(&req);

    let mut blocks: Vec<ExcludeBlock> = Vec::new();
    blocks.append(&mut vec![
        ExcludeBlock::Minutely,
        ExcludeBlock::Flags,
    ]);

    let forecast_builder = ForecastRequestBuilder::new(token, 38.636, -90.2399); //hardcoding lat & long for STL for now
    match context {
        "now" => {
            blocks.append(&mut vec![ExcludeBlock::Daily, ExcludeBlock::Hourly]);
        }
        "today" => {
            blocks.push(ExcludeBlock::Currently);
        }
        _ => (),
    }
    let forecast_req = forecast_builder.exclude_blocks(&mut blocks).build();
    match call.get_forecast(forecast_req.clone()) {
        Err(e) => format!("forecast error: {:?}", e),
        Ok(mut resp) => {
            let resp: ApiResponse = resp.json().unwrap();
            let mut s = String::with_capacity(80); //guessing at capacity
            if let Some(alerts) = resp.alerts {
                s.push_str(&format!("Alerts: {:?}\n", alerts))
            }

            //build for current
            if let Some(data) = resp.currently {
                if let Some(current) = data.summary {
                    s.push_str(&format!("Curently: {}\n", current))
                }
                if let Some(temp) = data.temperature {
                    s.push_str(&format!("Temp: {}\n", temp))
                }
                if let Some(gust) = data.wind_gust {
                    s.push_str(&format!("Wind gust: {}\n", gust))
                }
            }
            if let Some(data) = resp.hourly {
                if let Some(summary) = data.summary {
                    s.push_str(&format!("Today: {}\n", summary))
                }
            }
            //build data for daily
            if let Some(data) = resp.daily {
                if let Some(high) = data.data[0].temperature_high {
                    s.push_str(&format!("High: {}\n", high));
                }
                if let Some(low) = data.data[0].temperature_low {
                    s.push_str(&format!("Low: {}\n", low));
                }
                if let Some(sunrise) = data.data[0].sunrise_time {
                    let time = unix_to_local(sunrise);
                    s.push_str(&format!("Sunrise: {}:{} AM\n", time.hour() % 12, time.minute()));
                }
                if let Some(sunset) = data.data[0].sunset_time {
                    let time = unix_to_local(sunset);
                    s.push_str(&format!("Sunset: {}:{} PM", time.hour() % 12, time.minute()));
                }
            }
            s
        }
    }
}

fn unix_to_local(unix_time: u64) -> DateTime<Local> {
    let utc = Utc.timestamp(unix_time as i64, 0);
    utc.with_timezone(&Local)
}
