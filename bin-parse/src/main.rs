use arma3_wiki_github::report::Report;

mod command;
mod list;
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
        std::fs::create_dir(&tmp).unwrap();
    }

    let mut report = Report::new(version::version().await);

    command::commands(&mut report, &args, dry_run).await;

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
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    std::fs::write(&report_path, report_json).unwrap();
    println!("Report written to {report_path:?}");
}
