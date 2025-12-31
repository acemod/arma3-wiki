use arma3_wiki_github::report::Report;
use reqwest::{Client, RequestBuilder};

mod commands;
mod event_handlers;
mod version;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let args = args
        .iter()
        .filter(|arg| *arg != "--dry-run")
        .cloned()
        .collect::<Vec<_>>();
    let tmp = std::env::temp_dir().join("arma3-wiki-fetch");
    if !tmp.exists() {
        fs_err::create_dir(&tmp).expect("Failed to create temp directory");
    }

    let do_commands = !dry_run || args.is_empty() || args.iter().any(|arg| arg == "--commands");
    let do_event_handlers =
        !dry_run || args.is_empty() || args.iter().any(|arg| arg == "--event-handlers");

    println!("Dry run: {dry_run}");
    println!("Temp dir: {}", tmp.display());
    println!(
        "Doing commands: {}",
        if do_commands { "yes" } else { "no" }
    );
    println!(
        "Doing event handlers: {}",
        if do_event_handlers { "yes" } else { "no" }
    );

    let client = reqwest::Client::new();

    let mut report = Report::new(version::version(&client).await);

    if do_commands {
        print!("== Commands");
        commands::commands(&client, &mut report, &args, dry_run).await;

        for (command, errors) in report.failed_commands() {
            println!("Failed: {command}");
            for error in errors {
                println!("  {error}");
            }
        }

        println!("Passed:   {}", report.passed_commands().len());
        println!("Failed:   {}", report.failed_commands().len());
        println!("Outdated: {}", report.outdated_commands().len());
    }

    if do_event_handlers {
        println!("== EventHandlers");
        let _ = event_handlers::event_handlers(&client, &mut report, dry_run).await;

        println!("Passed:   {}", report.passed_event_handlers().len());
        println!("Failed:   {}", report.failed_event_handlers().len());
        println!("Outdated: {}", report.outdated_event_handlers().len());
    }

    // write report
    let report_path = tmp.join("report.json");
    let report_json = serde_json::to_string_pretty(&report).expect("Failed to serialize report");
    fs_err::write(&report_path, report_json).expect("Failed to write report");
    println!("Report written to {}", report_path.display());
}

trait WafSkip {
    fn bi_get(&self, url: &str) -> RequestBuilder;
}

impl WafSkip for Client {
    fn bi_get(&self, url: &str) -> RequestBuilder {
        self.get(url).header("User-Agent", "HEMTT Wiki Bot").header(
            "bi-waf-skip",
            std::env::var("BI_WAF_SKIP").expect("BI_WAF_SKIP not set"),
        )
    }
}
