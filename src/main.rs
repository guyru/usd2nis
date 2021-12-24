use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use regex::Regex;
use std::io::Read;
use structopt::StructOpt;

/// Convert from USD to NIS on a specified date.
#[derive(StructOpt, Debug)]
struct Cli {
    /// Conversion date
    date: String,

    /// USD amounts to convert
    #[structopt(name = "USD")]
    amount: Vec<f64>,
}

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_date(date: &String) -> chrono::ParseResult<NaiveDate> {
    let fmt = "%Y-%m-%d";
    Ok(NaiveDate::parse_from_str(date, fmt)?)
}

/// Gets the exchange date for a given date.
///
/// Retrieves the exchange date for a given date. If no exchange date was published, search
/// backwards up to 30 days.
///
/// # Examples
/// ```
/// ```
/// ```
/// ```
fn get_exchange_rate(date: NaiveDate) -> Result<(f64, NaiveDate)> {
    let max_date_retries = 30;
    let mut xchg_date = date.clone();

    let re = Regex::new(r"<RATE>(\d.\d+)</RATE>").unwrap();

    for _ in 0..max_date_retries {
        let request_url = format!(
            "https://www.boi.org.il/currency.xml?rdate={}&curr=01",
            xchg_date.format("%Y%m%d").to_string()
        );
        let mut res = reqwest::blocking::get(&request_url)
            .with_context(|| format!("Failed to retrieve {}", &request_url))?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;

        let cap = re.captures(&body);
        if cap.is_some() {
            let cap = cap.unwrap();
            return Ok((
                cap.get(1).unwrap().as_str().parse::<f64>().unwrap(),
                xchg_date,
            ));
        }
        xchg_date = xchg_date.pred();
    }
    bail!(format!("No conversion rate found for date {}", date))
}

fn main() -> Result<()> {
    let opt = Cli::from_args();

    let date = parse_date(&opt.date)?;
    let (rate, xchg_date) = get_exchange_rate(date)?;
    print!("{} ({}):", xchg_date, rate);
    for amount in opt.amount {
        print!(" {:.2}", rate * amount);
    }
    println!("");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_rate_weekday() -> Result<()> {
        let (rate, date) = get_exchange_rate(NaiveDate::from_ymd(2021, 12, 20))?;
        assert_eq!(rate, 3.152);
        assert_eq!(date, NaiveDate::from_ymd(2021, 12, 20));
        Ok(())
    }

    #[test]
    fn exchange_rate_sunday() -> Result<()> {
        let (rate, date) = get_exchange_rate(NaiveDate::from_ymd(2021, 12, 19))?;
        assert_eq!(rate, 3.115);
        assert_eq!(date, NaiveDate::from_ymd(2021, 12, 17));
        Ok(())
    }

    #[test]
    fn exchange_rate_distant_future() {
        assert!(get_exchange_rate(NaiveDate::from_ymd(3000, 01, 01)).is_err())
    }
}
