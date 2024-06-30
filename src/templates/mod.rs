pub mod monthly_tweets;
use regex::Regex;

/// Formatter for tweet text
struct Formatter {
    re_account: Regex,
    re_hash_number: Regex,
    re_hash_url: Regex,
}
impl Formatter {
    fn new() -> Self {
        Self {
            re_account: Regex::new(r"@([a-zA-Z0-9_]+)").unwrap(),
            re_hash_number: Regex::new(r"#(\d+)([「」『』（）【】:：｜\|]+)").unwrap(),
            re_hash_url: Regex::new(r"#(\d+)http").unwrap(),
        }
    }
    fn format_text(&self, text: &str) -> String {
        let mut text = text.replace("\n", "\n  ");
        text = self.re_account.replace_all(&text, r"[[@$1]]").to_string();
        text = self
            .re_hash_number
            .replace_all(&text, r"#$1 $2")
            .to_string();
        text = self.re_hash_url.replace_all(&text, r"#$1 http").to_string();
        text
    }
}
