use arma3_wiki_github::report::Report;
use reqwest::{Client, RequestBuilder};

mod commands;
mod event_handlers;
mod version;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let tmp = std::env::temp_dir().join("arma3-wiki-fetch");
    if !tmp.exists() {
        fs_err::create_dir(&tmp).expect("Failed to create temp directory");
    }

    println!("Temp dir: {}", tmp.display());

    let client = reqwest::Client::new();

    let mut report = Report::new(version::version(&client).await);

    print!("== Commands");
    report = commands::commands(&client, report, &args).await;

    for (command, errors) in report.failed_commands() {
        println!("Failed: {command}");
        for error in errors {
            println!("  {error}");
        }
    }

    println!("Passed:   {}", report.passed_commands().len());
    println!("Failed:   {}", report.failed_commands().len());
    println!("Outdated: {}", report.outdated_commands().len());

    // write report
    let report_path = tmp.join("report.json");
    let report_json = serde_json::to_string_pretty(&report).expect("Failed to serialize report");
    fs_err::write(&report_path, report_json).expect("Failed to write report");
    println!("Report written to {}", report_path.display());
}

trait WafSkip {
    fn bi_get(&self, url: &str) -> RequestBuilder;
    fn bi_head(&self, url: &str) -> RequestBuilder;
}

impl WafSkip for Client {
    fn bi_get(&self, url: &str) -> RequestBuilder {
        self.get(url).header("User-Agent", "HEMTT Wiki Bot").header(
            "bi-waf-skip",
            std::env::var("BI_WAF_SKIP").expect("BI_WAF_SKIP not set"),
        )
    }

    fn bi_head(&self, url: &str) -> RequestBuilder {
        self.head(url)
            .header("User-Agent", "HEMTT Wiki Bot")
            .header(
                "bi-waf-skip",
                std::env::var("BI_WAF_SKIP").expect("BI_WAF_SKIP not set"),
            )
    }
}
