/// A tool to convert Twitter data to Obsidian notes
use anyhow::Result;
use chrono::{Datelike, Months};
use clap::Parser;
use log::{error, info, warn};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};
use twitter2obsidian::{
    templates::monthly_tweets::{MonthlyTweetsTemplate, MonthlyTweetsTemplateInput},
    tweet::{parse_tweets, Tweet},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'f', long, help = "Path to the JSON file of tweet data")]
    tweets_file_path: String,
    #[arg(short = 'o', long, help = "Path to the output directory")]
    output_dir_path: String,
    #[arg(short = 's', long, help = "Start month to filter the tweets (YYYY-MM)")]
    start_month: Option<String>,
    #[arg(short = 'e', long, help = "End month to filter the tweets (YYYY-MM)")]
    end_month: Option<String>,
}

fn load_tweets(tweets_file_path: &str) -> Result<Vec<Tweet>> {
    info!("Loading tweets from {}", tweets_file_path);
    let file = match File::open(tweets_file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open the file {}: {}", tweets_file_path, e,);
            std::process::exit(1);
        }
    };
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    // Advance the reader to the first "[" character
    let content = content.trim_start_matches(|c| c != '[');

    parse_tweets(content)
}

fn filter_tweet_by_start_month(tweets: Vec<Tweet>, start_month: &str) -> Vec<Tweet> {
    info!("Filtering tweets by the start month: {}", start_month);
    let start_month = chrono::NaiveDate::parse_from_str(&format!("{}-01", start_month), "%Y-%m-%d")
        .expect("Failed to parse the start month");
    tweets
        .into_iter()
        .filter(|tweet| tweet.created_at().naive_local() >= start_month.into())
        .collect()
}
fn filter_tweet_by_end_month(tweets: Vec<Tweet>, end_month: &str) -> Vec<Tweet> {
    info!("Filtering tweets by the end month: {}", end_month);
    let mut end_month = chrono::NaiveDate::parse_from_str(&format!("{}-01", end_month), "%Y-%m-%d")
        .expect("Failed to parse the end month");
    // 翌月初日にする
    end_month = end_month
        .checked_add_months(Months::new(1))
        .expect("Failed to calculate the end month");
    tweets
        .into_iter()
        .filter(|tweet| tweet.created_at().naive_local() < end_month.into())
        .collect()
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let tweets = {
        let tweets = load_tweets(&args.tweets_file_path)?;
        // Filter the tweets by the start
        let tweets = match args.start_month {
            Some(ref start_month) => filter_tweet_by_start_month(tweets, start_month),
            None => tweets,
        };
        // Filter the tweets by the end
        let tweets = match args.end_month {
            Some(ref end_month) => filter_tweet_by_end_month(tweets, end_month),
            None => tweets,
        };
        tweets
    };

    let mut tweets_by_yyyymm = HashMap::new();
    for tweet in tweets.iter() {
        let dt = &tweet.created_at();
        let yyyymm = dt.year() * 100 + dt.month() as i32;
        tweets_by_yyyymm
            .entry(yyyymm)
            .or_insert_with(Vec::new)
            .push(tweet);
    }

    let template = MonthlyTweetsTemplate::new()?;

    for (yyyymm, tweets) in tweets_by_yyyymm.iter() {
        let data = match MonthlyTweetsTemplateInput::new(tweets) {
            Ok(data) => data,
            Err(e) => {
                warn!("Failed to create the template input for {}: {}", yyyymm, e);
                continue;
            }
        };

        let output_file_path = format!("{}/tweets_{}.md", args.output_dir_path, yyyymm);
        let mut output_file = match File::create(&output_file_path) {
            Ok(file) => file,
            Err(e) => {
                warn!("Failed to create the file({}): {}", output_file_path, e);
                continue;
            }
        };
        match template.render(&data, &mut output_file) {
            Ok(_) => {
                info!("Saved the tweets to {}", output_file_path)
            }
            Err(e) => {
                warn!("Failed to render the template for {}: {}", yyyymm, e);
            }
        }
    }

    Ok(())
}
