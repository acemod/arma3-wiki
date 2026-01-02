use arma3_wiki_replace::{CommunityDetails, WafSkip, temp};
use console::{Style, style};
use reqwest::Client;
use similar::ChangeTag;

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}
fn regex_contains(line: &str, is_regex: bool, search: &str) -> bool {
    if is_regex {
        let re = regex::Regex::new(search).expect("Invalid regex");
        re.is_match(line)
    } else {
        line.contains(search)
    }
}

// Replace $1, $2, etc. in the replace string with the corresponding capture groups from the regex
fn regex_replace(line: &str, is_regex: bool, search: &str, replace: &str) -> String {
    if is_regex {
        let re = regex::Regex::new(search).expect("Invalid regex");
        re.replace_all(line, replace).to_string()
    } else {
        line.replace(search, replace)
    }
}

#[tokio::main]
async fn main() {
    let is_regex = dialoguer::Confirm::new()
        .with_prompt("Use regex for search?")
        .interact()
        .unwrap();
    let search = dialoguer::Input::<String>::new()
        .with_prompt("Enter search term")
        .interact_text()
        .unwrap();
    let replace = dialoguer::Input::<String>::new()
        .with_prompt("Enter replace term")
        .interact_text()
        .unwrap();

    let details = CommunityDetails::load();
    println!("Loaded community details for user: {:?}", details);
    let client = reqwest::Client::builder()
        .cookie_provider(std::sync::Arc::new(details.to_cookies()))
        .build()
        .unwrap();

    for file in temp().read_dir().unwrap() {
        let file = file.unwrap().path();
        if !file.is_file() {
            continue;
        }
        let content = tokio::fs::read_to_string(&file).await.unwrap();
        if regex_contains(&content, is_regex, &search) {
            let content = fetch_command(&client, file.file_name().unwrap().to_str().unwrap())
                .await
                .unwrap(); // Refetch the content, in case it has changed
            tokio::fs::write(&file, &content).await.unwrap(); // Update the temp file
            if !regex_contains(&content, is_regex, &search) {
                println!(
                    "File {} no longer contains search term, skipping",
                    file.display()
                );
                continue;
            }
            let new_content = regex_replace(&content, is_regex, &search, &replace);
            // show diff with similar crate
            let diff = similar::TextDiff::from_lines(&content, &new_content);
            println!("Changes in file: {}", file.display());
            for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
                if idx > 0 {
                    println!("{:-^1$}", "-", 80);
                }
                for op in group {
                    for change in diff.iter_inline_changes(op) {
                        let (sign, s) = match change.tag() {
                            ChangeTag::Delete => ("-", Style::new().red()),
                            ChangeTag::Insert => ("+", Style::new().green()),
                            ChangeTag::Equal => (" ", Style::new().dim()),
                        };
                        print!(
                            "{}{} |{}",
                            style(Line(change.old_index())).dim(),
                            style(Line(change.new_index())).dim(),
                            s.apply_to(sign).bold(),
                        );
                        for (emphasized, value) in change.iter_strings_lossy() {
                            if emphasized {
                                print!("{}", s.apply_to(value).underlined().on_black());
                            } else {
                                print!("{}", s.apply_to(value));
                            }
                        }
                        if change.missing_newline() {
                            println!();
                        }
                    }
                }
            }
            println!();
            if !dialoguer::Confirm::new()
                .with_prompt("Apply these changes?")
                .interact()
                .unwrap()
            {
                continue;
            }
            println!("Applying changes to file: {}", file.display());
            submit_command(
                &client,
                file.file_name().unwrap().to_str().unwrap(),
                &new_content,
            )
            .await
            .unwrap();
            tokio::fs::write(&file, new_content).await.unwrap();
        }
    }
}

async fn fetch_command(client: &Client, command: &str) -> Result<String, String> {
    let raw_url = format!("https://community.bistudio.com/wiki/{command}?action=raw");
    let res = match client.bi_get(&raw_url).send().await {
        Ok(res) => res,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    assert!(
        res.status().is_success(),
        "Failed to fetch {command}: {}",
        res.status()
    );
    let content = res.text().await.expect("Failed to read response text");
    if content.is_empty() {
        return Err("Empty".to_string());
    }
    let path = temp().join(command);
    let mut file = tokio::fs::File::create(&path)
        .await
        .expect("Failed to create temp file");
    tokio::io::AsyncWriteExt::write_all(&mut file, content.as_bytes())
        .await
        .expect("Failed to write to temp file");
    Ok(content)
}

async fn get_edit_token(client: &Client, command: &str) -> Result<(String, String), String> {
    fn extract(key: &str, text: &str) -> Option<String> {
        let (first_half, _) = text.split_once(&format!(r#"" name="{key}"/>"#))?;
        let (_, token) = first_half.rsplit_once(r#"value=""#)?;
        Some(token.to_string())
    }
    let url = format!(
        "https://community.bistudio.com/wiki/{}?action=edit",
        command
    );
    let res = match client.bi_get(&url).send().await {
        Ok(res) => res,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    assert!(
        res.status().is_success(),
        "Failed to get edit token: {}",
        res.status()
    );
    // Look in the page for <input type="hidden" value="e19941ee1a6c7ec2eab083fa55a3f75e69576143+\" name="wpEditToken"/>
    let text = res.text().await.expect("Failed to read response text");
    std::fs::write("debug_edit_page.html", text.clone()).unwrap();
    let token = extract("wpEditToken", &text).ok_or("Failed to extract edit token")?;
    let edit_rev_id = extract("editRevId", &text).ok_or("Failed to extract editRevId")?;
    println!("Extracted editToken: {}, editRevId: {}", token, edit_rev_id);
    Ok((token, edit_rev_id))
}

async fn submit_command(client: &Client, command: &str, content: &str) -> Result<(), String> {
    // send a post to the mediawiki api.php to edit the page
    let (token, edit_rev_id) = get_edit_token(client, command).await?;
    let enc_token = urlencoding::encode(&token);
    let enc_command = urlencoding::encode(command);
    let enc_content = urlencoding::encode(content);
    let enc_edit_rev_id = urlencoding::encode(&edit_rev_id);
    let url = "https://community.bistudio.com/wikidata/api.php".to_string();
    let res = match client
        .bi_post(&url)
        .body(format!("action=edit&format=json&formatversion=2&title={enc_command}&section=0&summary=&minor=true&contentmodel=wikitext&contentformat=text%2Fx-wiki&baserevid={enc_edit_rev_id}&text={enc_content}&token={enc_token}"))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    assert!(
        res.status().is_success(),
        "Failed to submit {command}: {}",
        res.status()
    );
    println!("Submitted changes to {}", command);
    std::fs::write("debug_submit_response.html", res.text().await.unwrap()).unwrap();
    Ok(())
}

fn change_body(token: &str, content: &str) -> String {
    format!(
        r##"------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpUnicodeCheck"

ℳ𝒲♥𝓊𝓃𝒾𝒸ℴ𝒹ℯ
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpAntispam"


------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wikieditorUsed"

yes
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpSection"


------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpStarttime"

20260102061257
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpEdittime"

20240616083856
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="editRevId"

364811
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpScrolltop"

314
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpAutoSummary"

d41d8cd98f00b204e9800998ecf8427e
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="oldid"

0
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="parentRevId"

364811
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="format"

text/x-wiki
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="model"

wikitext
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpTextbox1"

{content}
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpSummary"


------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpMinoredit"


------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpWatchthis"


------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpSave"

Save changes
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpEditToken"

{token}
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="mode"

text
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d
Content-Disposition: form-data; name="wpUltimateParam"

1
------geckoformboundary8785cc78bdaa2cdba9d2ee81622df07d--"##
    )
}
