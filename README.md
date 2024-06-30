# twitter2obsidian

Twitter2obsidian is a tool that converts tweet data into Obsidian-friendly markdown format. It is written in Rust.

## Usage

Execute the following command to generate files grouped by month in the output directory. Be careful that the files will be overwritten if they already exist.

```sh
twitter2obsidian [OPTIONS] --tweets-file-path <TWEETS_FILE_PATH> --output-dir-path <OUTPUT_DIR_PATH>

Options:
  -f, --tweets-file-path <TWEETS_FILE_PATH>  Path to the JSON file of tweet data
  -o, --output-dir-path <OUTPUT_DIR_PATH>    Path to the output directory
  -s, --start-month <START_MONTH>            Start month to filter the tweets (YYYY-MM)
  -e, --end-month <END_MONTH>                End month to filter the tweets (YYYY-MM)
  -h, --help                                 Print help
  -V, --version                              Print version
```
