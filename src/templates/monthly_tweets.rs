use super::Formatter;
use crate::tweet::Tweet;
use anyhow::Result;
use chrono::{DateTime, Datelike, Local, Timelike};
use handlebars::Handlebars;
use log::error;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, PartialEq)]
struct TweetCountByHour {
    hour: usize,
    tweet_count: usize,
    retweet_count: usize,
    reply_count: usize,
}
impl TweetCountByHour {
    fn new(hour: usize) -> Self {
        Self {
            hour,
            tweet_count: 0,
            retweet_count: 0,
            reply_count: 0,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
struct ActivityStats {
    tweet_count: usize,
    retweet_count: usize,
    reply_count: usize,
    tweet_count_by_hour: Vec<TweetCountByHour>,
}
#[derive(Debug, Serialize)]
struct FormattedTweet {
    created_at: String,
    text: String,
}

/// input data for the monthly_tweets template
#[derive(Debug, Serialize)]
pub struct MonthlyTweetsTemplateInput {
    id: String,
    file_created_at: String,
    month: String,
    year: String,
    stats: ActivityStats,
    tweets: Vec<FormattedTweet>,
}

impl MonthlyTweetsTemplateInput {
    fn format_tweets(tweets: &[&Tweet]) -> Vec<FormattedTweet> {
        let formatter = Formatter::new();
        let mut formatted_tweets = tweets
            .iter()
            .map(|tw| FormattedTweet {
                created_at: tw.created_at().format("%Y-%m-%d %H:%M:%S").to_string(),
                text: formatter.format_text(tw.full_text()),
            })
            .collect::<Vec<FormattedTweet>>();
        formatted_tweets.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        formatted_tweets
    }
    fn extract_earliest_tweet_created_at(tweets: &[&Tweet]) -> DateTime<Local> {
        let first_tweet = tweets
            .iter()
            .min_by(|a, b| a.created_at().cmp(&b.created_at()))
            .unwrap();
        first_tweet.created_at()
    }
    fn format_id(created_at: &DateTime<Local>) -> String {
        created_at.format("%Y%m%d%H%M%S%3f").to_string()
    }
    fn format_file_created_at(created_at: &DateTime<Local>) -> String {
        created_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    fn generate_activity_stats(tweets: &[&Tweet]) -> ActivityStats {
        let mut tweet_count_by_hour = [0; 24]
            .iter()
            .enumerate()
            .map(|(i, _)| TweetCountByHour::new(i))
            .collect::<Vec<TweetCountByHour>>();
        for tweet in tweets.iter() {
            let created_at = tweet.created_at();
            let hour = created_at.hour() as usize;
            tweet_count_by_hour[hour].tweet_count += 1;
            if tweet.is_retweet() {
                tweet_count_by_hour[hour].retweet_count += 1;
            }
            if tweet.is_reply() {
                tweet_count_by_hour[hour].reply_count += 1;
            }
        }
        let tweet_count = tweets.len();
        let retweet_count = tweets.iter().filter(|tw| tw.is_retweet()).count();
        let reply_count = tweets.iter().filter(|tw| tw.is_reply()).count();
        ActivityStats {
            tweet_count,
            retweet_count,
            reply_count,
            tweet_count_by_hour,
        }
    }

    /// create a new MonthlyTweetsTemplateInput from the given tweets
    pub fn new(tweets: &[&Tweet]) -> Result<Self> {
        let (year, month, id, file_created_at) = {
            let earliest_tweet_created_at = Self::extract_earliest_tweet_created_at(tweets);
            (
                earliest_tweet_created_at.year().to_string(),
                format!("{:02}", earliest_tweet_created_at.month()),
                Self::format_id(&earliest_tweet_created_at),
                Self::format_file_created_at(&earliest_tweet_created_at),
            )
        };
        let stats = Self::generate_activity_stats(tweets);
        let formatted_tweets = Self::format_tweets(tweets);

        Ok(Self {
            id,
            file_created_at,
            month,
            year,
            stats,
            tweets: formatted_tweets,
        })
    }
}
/// A struct representing the monthly_tweets template
pub struct MonthlyTweetsTemplate<'a> {
    handlebars: Handlebars<'a>,
}
impl<'a> MonthlyTweetsTemplate<'a> {
    const TEMPLATE_NAME: &'static str = "monthly_tweets";
    /// Create a new MonthlyTweetsTemplate
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        let tpl_path = MonthlyTweetsTemplate::get_template_path();
        if let Err(e) = handlebars.register_template_file(Self::TEMPLATE_NAME, &tpl_path) {
            error!(
                "Failed to register the template file {}: {}",
                tpl_path.display(),
                e
            );
            std::process::exit(1);
        }
        Ok(Self { handlebars })
    }

    fn get_template_path() -> PathBuf {
        let current_file_path = Path::new(file!());
        let current_file_dir = current_file_path.parent().unwrap();
        current_file_dir
            .join(Self::TEMPLATE_NAME)
            .with_extension("hbs")
    }

    /// Render file with the given input
    pub fn render(&self, input: &MonthlyTweetsTemplateInput, file: &mut File) -> Result<()> {
        self.handlebars
            .render_to_write(Self::TEMPLATE_NAME, &input, file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    #[test]
    fn test_get_template_path() {
        let path = super::MonthlyTweetsTemplate::get_template_path();
        assert!(path.exists());
    }
    #[test]
    fn test_format_id() {
        let created_at = chrono::Local
            .with_ymd_and_hms(2023, 3, 11, 4, 12, 48)
            .unwrap();
        let id = super::MonthlyTweetsTemplateInput::format_id(&created_at);
        assert_eq!(id, "20230311041248000");
    }
    #[test]
    fn test_format_file_created_at() {
        let created_at = chrono::Local
            .with_ymd_and_hms(2023, 3, 11, 4, 12, 48)
            .unwrap();
        let file_created_at =
            super::MonthlyTweetsTemplateInput::format_file_created_at(&created_at);
        assert_eq!(file_created_at, "2023-03-11 04:12:48");
    }
    #[test]
    fn test_generate_activity_stats() {
        let tweet1 = super::Tweet::new_with_local_datetime(
            chrono::Local
                .with_ymd_and_hms(2023, 3, 15, 0, 12, 48)
                .unwrap(),
            "tweet1".to_string(),
            false,
        );
        let tweet2 = super::Tweet::new_with_local_datetime(
            chrono::Local
                .with_ymd_and_hms(2023, 3, 12, 2, 12, 48)
                .unwrap(),
            "RT @hoge: tweet2".to_string(),
            false,
        );
        let tweet3 = super::Tweet::new_with_local_datetime(
            chrono::Local
                .with_ymd_and_hms(2023, 3, 14, 23, 12, 48)
                .unwrap(),
            "@hoge tweet3".to_string(),
            true,
        );
        let actual = super::MonthlyTweetsTemplateInput::generate_activity_stats(&vec![
            &tweet1, &tweet2, &tweet3,
        ]);
        let expected = super::ActivityStats {
            tweet_count: 3,
            retweet_count: 1,
            reply_count: 1,
            tweet_count_by_hour: vec![
                super::TweetCountByHour {
                    hour: 0,
                    tweet_count: 1,
                    retweet_count: 0,
                    reply_count: 0,
                },
                super::TweetCountByHour::new(1),
                super::TweetCountByHour {
                    hour: 2,
                    tweet_count: 1,
                    retweet_count: 1,
                    reply_count: 0,
                },
                super::TweetCountByHour::new(3),
                super::TweetCountByHour::new(4),
                super::TweetCountByHour::new(5),
                super::TweetCountByHour::new(6),
                super::TweetCountByHour::new(7),
                super::TweetCountByHour::new(8),
                super::TweetCountByHour::new(9),
                super::TweetCountByHour::new(10),
                super::TweetCountByHour::new(11),
                super::TweetCountByHour::new(12),
                super::TweetCountByHour::new(13),
                super::TweetCountByHour::new(14),
                super::TweetCountByHour::new(15),
                super::TweetCountByHour::new(16),
                super::TweetCountByHour::new(17),
                super::TweetCountByHour::new(18),
                super::TweetCountByHour::new(19),
                super::TweetCountByHour::new(20),
                super::TweetCountByHour::new(21),
                super::TweetCountByHour::new(22),
                super::TweetCountByHour {
                    hour: 23,
                    tweet_count: 1,
                    retweet_count: 0,
                    reply_count: 1,
                },
            ],
        };

        for (actual, expected) in actual
            .tweet_count_by_hour
            .iter()
            .zip(expected.tweet_count_by_hour.iter())
        {
            assert_eq!(actual, expected);
        }
        assert_eq!(actual.tweet_count, expected.tweet_count);
        assert_eq!(actual.retweet_count, expected.retweet_count);
        assert_eq!(actual.reply_count, expected.reply_count);
    }
}
