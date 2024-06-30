use anyhow::Result;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A struct representing a tweet
#[derive(Debug, Deserialize, Serialize)]
pub struct Tweet {
    created_at: DateTime<Local>,
    full_text: String,
    is_reply: bool,
}
impl Tweet {
    pub fn new(created_at: String, full_text: String, is_reply: bool) -> Result<Self> {
        Ok(Self {
            created_at: parse_twitter_date(&created_at)?.with_timezone(&Local),
            full_text,
            is_reply,
        })
    }
    pub fn created_at(&self) -> DateTime<Local> {
        self.created_at
    }
    pub fn full_text(&self) -> &str {
        &self.full_text
    }
    pub fn is_reply(&self) -> bool {
        self.is_reply
    }
    pub fn is_retweet(&self) -> bool {
        self.full_text.starts_with("RT @")
    }
    #[cfg(test)]
    pub fn new_with_local_datetime(
        created_at: DateTime<Local>,
        full_text: String,
        is_reply: bool,
    ) -> Self {
        Self {
            created_at,
            full_text,
            is_reply,
        }
    }
}

/// Parse JSON formatted tweets and return a vector of Tweet
pub fn parse_tweets(tweets: &str) -> Result<Vec<Tweet>> {
    let data: Vec<Value> = serde_json::from_str(tweets).expect("Failed to parse JSON data");
    data.iter()
        .map(|tw| {
            Tweet::new(
                tw["tweet"]["created_at"].as_str().unwrap().to_string(),
                tw["tweet"]["full_text"].as_str().unwrap().to_string(),
                !tw["tweet"]["in_reply_to_user_id"].is_null(),
            )
        })
        .collect()
}

/// Parse a Twitter formatted date string and return a DateTime<Utc>
fn parse_twitter_date(date: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let dt = DateTime::parse_from_str(date, "%a %b %d %H:%M:%S %z %Y")?;
    Ok(dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_twitter_date() {
        let date = "Sat Mar 11 04:12:48 +0000 2023";
        let expected = Utc.with_ymd_and_hms(2023, 3, 11, 4, 12, 48).unwrap();
        assert_eq!(parse_twitter_date(date), Ok(expected));
    }
}
