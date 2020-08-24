///
///
// TODO: timeout to milliseconds
mod args;
mod ports;
mod readin;

use futures::{stream, StreamExt};
use std::time::Duration;

use args::Arguments;
use ports::ProtoPort;

use log::{info, warn};
use reqwest::{header, redirect, Client, Response};
use scraper::{Html, Selector};
use stderrlog;

fn init_log(verbosity: usize) {
    stderrlog::new()
        .module(module_path!())
        .verbosity(verbosity)
        .init()
        .expect("Error initiating log");
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse_args();
    init_log(args.verbosity);
    let client = create_http_client(args.timeout, args.follow_redirect);
    let invalid_codes = &args.invalid_codes;
    let show_status = args.show_status;
    let show_title = args.show_title;
    let delimiter = &args.delimiter;
    let fetches = stream::iter(gen_urls(args.proto_ports, args.targets))
        .map(|url| {
            let client = &client;
            async move {
                info!("Request {}", url);
                match client.get(&url).send().await {
                    Ok(resp) => {
                        if invalid_codes.contains(&resp.status().as_u16()) {
                            warn!("{}: 400 Response", resp.url());
                        } else {
                            let mut message = vec![format!("{}", url)];
                            if show_status {
                                message.push(format!(
                                    "{}",
                                    resp.status().as_u16()
                                ));
                            }

                            if show_title {
                                let title = get_resp_title(resp)
                                    .await
                                    .unwrap_or("".to_string());
                                let title = title.replace("\n", "");
                                message.push(title);
                            }
                            println!("{}", message.join(delimiter));
                        }
                    }
                    Err(err) => {
                        warn!("{}", err);
                    }
                };
            }
        })
        .buffer_unordered(args.workers)
        .collect::<Vec<()>>();

    fetches.await;
}

fn create_http_client(timeout: u64, redirect: bool) -> Client {
    let redirect_policy = match redirect {
        true => redirect::Policy::limited(3),
        false => redirect::Policy::none(),
    };

    let builder = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(redirect_policy)
        .timeout(Duration::from_millis(timeout));

    return builder.build().unwrap();
}

async fn get_resp_title(resp: Response) -> Option<String> {
    if is_html_resp(&resp) {
        match resp.text().await {
            Ok(text) => {
                return extract_html_title(&text);
            }
            Err(_) => {}
        }
    }

    return None;
}

fn is_html_resp(resp: &Response) -> bool {
    let headers = resp.headers();
    let content_type = headers.get(header::CONTENT_TYPE);

    if let Some(content_type) = content_type {
        let content_type =
            content_type.to_str().expect("Error parsing content-type");

        let content_type: &str =
            content_type.split(";").collect::<Vec<&str>>()[0];

        match content_type {
            "text/html" | "text/xml" | "application/xhtml+xml" => {
                return true;
            }
            _ => {}
        }
    }

    return false;
}

fn extract_html_title(html: &str) -> Option<String> {
    let html = Html::parse_document(html);
    let selector = Selector::parse("title").unwrap();

    for tag in html.select(&selector) {
        return Some(tag.text().collect::<Vec<_>>().join(""));
    }

    return None;
}

/// Function to generate urls, from protocol,
/// host and port and returns them in a iterator.
fn gen_urls(
    proto_ports: Vec<ProtoPort>,
    targets: Vec<String>,
) -> impl Iterator<Item = String> {
    return UrlGenerator::new(
        proto_ports,
        Box::new(readin::read_inputs(targets)),
    );
}

struct UrlGenerator {
    proto_ports: Vec<ProtoPort>,
    targets: Box<dyn Iterator<Item = String>>,
    current_target: Option<String>,
    current_proto_ports: Vec<ProtoPort>,
}

impl UrlGenerator {
    fn new(
        proto_ports: Vec<ProtoPort>,
        targets: Box<dyn Iterator<Item = String>>,
    ) -> Self {
        Self {
            proto_ports: proto_ports,
            targets,
            current_target: None,
            current_proto_ports: Vec::new(),
        }
    }
}

impl Iterator for UrlGenerator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            match self.current_proto_ports.pop() {
                Some(pp) => {
                    return Some(format!(
                        "{}://{}:{}",
                        pp.proto,
                        self.current_target.as_ref().unwrap(),
                        pp.port
                    ));
                }
                None => {
                    self.current_target = Some(self.targets.next()?);
                    self.current_proto_ports = self.proto_ports.clone();
                    self.current_proto_ports.reverse();
                }
            }
        }
    }
}
