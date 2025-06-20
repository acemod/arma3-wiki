use std::{collections::HashMap, path::Path};

use arma3_wiki::model::{EventHandler, EventHandlerNamespace, ParsedEventHandler};
use arma3_wiki_github::report::Report;
use reqwest::Client;

use crate::WafSkip;

#[allow(clippy::too_many_lines)]
pub async fn event_handlers(
    client: &Client,
    report: &mut Report,
    dry_run: bool,
) -> HashMap<EventHandlerNamespace, Vec<EventHandler>> {
    const URL: &str = "https://community.bistudio.com/wiki?title=Arma_3:_Event_Handlers&action=raw";
    let tmp = std::env::temp_dir()
        .join("arma3-wiki-fetch")
        .join("eventhandler_main.html");

    let body: String = if tmp.exists() {
        std::fs::read_to_string(&tmp).unwrap()
    } else {
        let request = client.bi_get(URL).send().await.unwrap();
        assert!(
            request.status().is_success(),
            "Failed to fetch event handlers list"
        );
        let content = request.text().await.unwrap();
        std::fs::write(&tmp, &content).unwrap();
        content
    };

    println!("Body length: {}", body.len());

    let mut event_handlers: HashMap<EventHandlerNamespace, Vec<EventHandler>> = HashMap::new();

    let mut section = None;
    let mut recording = false;
    let mut buffer = String::new();

    let lines = body.lines();

    let headings = vec![
        (EventHandlerNamespace::Standard, "=== Standard ==="),
        (
            EventHandlerNamespace::Multiplayer,
            "== Multiplayer Event Handlers ==",
        ),
        (
            EventHandlerNamespace::Mission,
            "== Mission Event Handlers ==",
        ),
        (
            EventHandlerNamespace::UserAction,
            "{{ArgTitle|2|UserAction Event Handlers|{{GVI|arma3|2.06}}}}",
        ),
        (
            EventHandlerNamespace::Projectile,
            "{{ArgTitle|2|Projectile Event Handlers|{{GVI|arma3|2.10}}}}",
        ),
        (
            EventHandlerNamespace::Group,
            "{{ArgTitle|2|Group Event Handlers|{{GVI|arma3|2.10}}}}",
        ),
        (
            EventHandlerNamespace::UserInterface,
            "== UI Event Handlers (Displays and Controls) ==",
        ),
        (EventHandlerNamespace::Music, "== Music Event Handlers =="),
        (EventHandlerNamespace::Eden, "== Eden Editor =="),
    ];

    'line: for line in lines {
        match section {
            None => {
                if line == "=== Standard ===" {
                    section = Some(EventHandlerNamespace::Standard);
                }
            }
            Some(ns) => {
                for (ns, heading) in &headings {
                    if &line == heading {
                        section = Some(*ns);
                        recording = false;
                        if !buffer.is_empty() && !buffer.contains("{{ConfigPage|abc}}") {
                            match ParsedEventHandler::from_wiki(&buffer) {
                                Ok(event_handler) => {
                                    event_handlers
                                        .entry(*ns)
                                        .or_default()
                                        .push(EventHandler::Parsed(event_handler));
                                }
                                Err((name, e)) => {
                                    eprintln!("Failed to parse event handler: {e}");
                                    event_handlers
                                        .entry(*ns)
                                        .or_default()
                                        .push(EventHandler::Failed(name, e));
                                }
                            }
                        }
                        buffer.clear();
                        continue 'line;
                    }
                }

                if line.starts_with("===")
                    || line.starts_with("{{ArgTitle|4|")
                    || line.starts_with("{{ConfigPage|end}}")
                {
                    if recording {
                        match ParsedEventHandler::from_wiki(&buffer) {
                            Ok(event_handler) => {
                                event_handlers
                                    .entry(ns)
                                    .or_default()
                                    .push(EventHandler::Parsed(event_handler));
                            }
                            Err((name, e)) => {
                                eprintln!("Failed to parse event handler: {e}");
                                event_handlers
                                    .entry(ns)
                                    .or_default()
                                    .push(EventHandler::Failed(name, e));
                            }
                        }
                    }
                    buffer.clear();
                    recording = line.starts_with("====") || line.starts_with("{{ArgTitle|4|");
                }

                if recording {
                    buffer.push_str(line);
                    buffer.push('\n');
                }
            }
        }
    }

    event_handlers.insert(
        EventHandlerNamespace::Eden,
        subsection(
            client,
            "https://community.bistudio.com/wiki/Arma_3:_Eden_Editor_Event_Handlers?action=raw",
            "eden",
            None,
            Some("== Object Event Handlers ==".to_owned()),
        )
        .await,
    );
    event_handlers.insert(
        EventHandlerNamespace::Standard,
        subsection(
            client,
            "https://community.bistudio.com/wiki/Arma_3:_Eden_Editor_Event_Handlers?action=raw",
            "eden",
            Some("== Object Event Handlers ==".to_owned()),
            None,
        )
        .await,
    );
    event_handlers.insert(
        EventHandlerNamespace::UserInterface,
        subsection(
            client,
            "https://community.bistudio.com/wiki/User_Interface_Event_Handlers?action=raw",
            "ui",
            None,
            None,
        )
        .await
        .into_iter()
        .map(|eh| match eh {
            EventHandler::Parsed(mut eh) => {
                eh.set_id(eh.id().trim_start_matches("on").to_string());
                EventHandler::Parsed(eh)
            }
            EventHandler::Failed(name, e) => EventHandler::Failed(name, e),
        })
        .collect(),
    );
    event_handlers.insert(
        EventHandlerNamespace::Mission,
        subsection(
            client,
            "https://community.bistudio.com/wiki/Arma_3:_Mission_Event_Handlers?action=raw",
            "mission",
            None,
            None,
        )
        .await,
    );

    for (ns, handlers) in &event_handlers {
        for handler in handlers {
            match &handler {
                EventHandler::Failed(_, _) => {
                    report.add_failed_event_handler(*ns, handler.clone());
                }
                EventHandler::Parsed(handler) => {
                    let mut dist_path = Path::new("./dist/events")
                        .join(urlencoding::encode(&ns.to_string()).to_string())
                        .join(handler.id());
                    dist_path.set_extension("yml");
                    let mut write = true;
                    if dist_path.exists() {
                        // Check if the file has changed
                        let old = std::fs::read_to_string(&dist_path).unwrap();
                        if old == serde_yaml::to_string(&handler).unwrap() {
                            write = false;
                            report.add_outdated_event_handler(*ns, handler.clone());
                        }
                    }
                    if write {
                        report.add_passed_event_handler(*ns, handler.clone());
                    }
                    if !dry_run && write {
                        if !dist_path.parent().expect("parent").exists() {
                            std::fs::create_dir_all(dist_path.parent().expect("parent")).unwrap();
                        }
                        let mut file = tokio::fs::File::create(dist_path).await.unwrap();
                        tokio::io::AsyncWriteExt::write_all(
                            &mut file,
                            serde_yaml::to_string(&handler).unwrap().as_bytes(),
                        )
                        .await
                        .unwrap();
                    }
                }
            }
        }
    }

    event_handlers
}

async fn subsection(
    client: &Client,
    url: &str,
    tag: &str,
    get_from: Option<String>,
    get_to: Option<String>,
) -> Vec<EventHandler> {
    let tmp = std::env::temp_dir()
        .join("arma3-wiki-fetch")
        .join(format!("eventhandler_{tag}.html"));

    let mut body: String = if tmp.exists() {
        std::fs::read_to_string(&tmp).unwrap()
    } else {
        let request = client.bi_get(url).send().await.unwrap();
        assert!(
            request.status().is_success(),
            "Failed to fetch event handlers list"
        );
        let content = request.text().await.unwrap();
        std::fs::write(&tmp, &content).unwrap();
        content
    };

    if get_from.is_some() {
        body = body.split_once(&get_from.unwrap()).unwrap().1.to_owned();
    }
    if get_to.is_some() {
        body = body.split_once(&get_to.unwrap()).unwrap().0.to_owned();
    }

    let mut event_handlers = Vec::new();

    let mut recording = false;
    let mut buffer = String::new();

    for line in body.lines() {
        if line.starts_with("===")
            || line.starts_with("{{ArgTitle|4|")
            || line.starts_with("{{ConfigPage|end}}")
        {
            if recording {
                match ParsedEventHandler::from_wiki(&buffer) {
                    Ok(event_handler) => {
                        event_handlers.push(EventHandler::Parsed(event_handler));
                    }
                    Err((name, e)) => {
                        eprintln!("Failed to parse event handler: {e}");
                        event_handlers.push(EventHandler::Failed(name, e));
                    }
                }
                buffer.clear();
            }
            recording = line.starts_with("====") || line.starts_with("{{ArgTitle|4|");
        }
        if recording {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }

    if !buffer.is_empty() {
        match ParsedEventHandler::from_wiki(&buffer) {
            Ok(event_handler) => {
                event_handlers.push(EventHandler::Parsed(event_handler));
            }
            Err((name, e)) => {
                eprintln!("Failed to parse event handler: {e}");
                event_handlers.push(EventHandler::Failed(name, e));
            }
        }
    }

    event_handlers
}
