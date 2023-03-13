use anyhow::{bail, Context, Result};
use chrono::{Duration, NaiveDate};
use clap::Parser;
use regex::Regex;
use std::{io::Read, thread};

/// Convert from USD to NIS on a specified date.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Conversion date
    date: String,

    /// USD amounts to convert
    #[arg(name = "USD", required = true)]
    amount: Vec<f64>,
}

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_date(date: &str) -> chrono::ParseResult<NaiveDate> {
    let fmt = "%Y-%m-%d";
    NaiveDate::parse_from_str(date, fmt)
}

/// Gets the exchange date for a given date.
///
/// Retrieves the exchange date for a given date. If no exchange date was published, search
/// backwards up to 30 days.
fn get_exchange_rate(date: NaiveDate) -> Result<(f64, NaiveDate)> {
    let start_date = date - Duration::days(30);

    let request_url = format!(
        // Query url constructed using https://edge.boi.gov.il/FusionDataBrowser/
        "https://edge.boi.gov.il/FusionEdgeServer/sdmx/v2/data/dataflow/BOI.STATISTICS/EXR/1.0/RER_USD_ILS.D.USD.ILS.ILS.OF00?c%5BTIME_PERIOD%5D=ge:{}+le:{}&locale=en",
        start_date.format("%Y-%m-%d"),
        date.format("%Y-%m-%d")
    );

    // We match the last date available in the series
    let re = Regex::new(
        r#"<Obs TIME_PERIOD="(\d{4}-\d{2}-\d{2})" OBS_VALUE="(\d+\.\d+)"[^>]*></Obs></Series>"#,
    )
    .unwrap();

    let mut res = 'req: {
        for _ in 0..3 {
            let result = reqwest::blocking::get(&request_url);
            if result.is_ok() {
                break 'req result;
            }
            thread::sleep(std::time::Duration::from_millis(500));
        }
        reqwest::blocking::get(&request_url)
    }
    .with_context(|| format!("Failed to retrieve {}", &request_url))?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    if let Some(cap) = re.captures(&body) {
        return Ok((
            cap.get(2).unwrap().as_str().parse::<f64>().unwrap(),
            NaiveDate::parse_from_str(cap.get(1).unwrap().as_str(), "%Y-%m-%d").unwrap(),
        ));
    };
    bail!(format!("No conversion rate found for date {}", date))
}

fn main() -> Result<()> {
    let opt = Cli::parse();

    let date = parse_date(&opt.date)?;
    let (rate, xchg_date) = get_exchange_rate(date)?;
    print!("{} ({}):", xchg_date, rate);
    for amount in opt.amount {
        print!(" {:.2}", rate * amount);
    }
    println!();

    Ok(())
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_rate_weekday() -> Result<()> {
        let (rate, date) = get_exchange_rate(NaiveDate::from_ymd_opt(2021, 12, 20).unwrap())?;
        assert_eq!(rate, 3.152);
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 12, 20).unwrap());
        Ok(())
    }

    #[test]
    fn exchange_rate_sunday() -> Result<()> {
        let (rate, date) = get_exchange_rate(NaiveDate::from_ymd_opt(2021, 12, 19).unwrap())?;
        assert_eq!(rate, 3.115);
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 12, 17).unwrap());
        Ok(())
    }

    #[test]
    fn exchange_rate_distant_future() {
        assert!(get_exchange_rate(NaiveDate::from_ymd_opt(3000, 01, 01).unwrap()).is_err())
    }
}
