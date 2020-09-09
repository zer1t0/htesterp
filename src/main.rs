///
///
mod args;
mod ports;
mod readin;

use futures::{stream, StreamExt};
use futures::future;
use serde::Serialize;
use std::time::Duration;

use args::Arguments;
use ports::ProtoPort;

use log::{info, warn, error};
use reqwest::{header, redirect, Client, Response};
use scraper::{Html, Selector};
use stderrlog;

use std::fs::File;
use std::io::Write;

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
                return process_url(url, client, invalid_codes, show_status, show_title, delimiter).await;
            }
        })
        .buffer_unordered(args.workers)
        .filter(|r| future::ready(r.is_some())).map(|r| r.unwrap())
        .collect::<Vec<ResponseInfo>>();

    let results = fetches.await;
    if let Some(json_file) = args.json_file {
        info!("Saving json result in {}", json_file);
        save_json(json_file, &results);
    }
}

async fn process_url(
    url: String,
    client: &Client,
    invalid_codes: &Vec<u16>,
    show_status: bool,
    show_title: bool,
    delimiter: &str
) -> Option<ResponseInfo> {
    info!("Request {}", url);
    match client.get(&url).send().await {
        Ok(resp) => {
            let resp_info = parse_response(resp).await;
            if invalid_codes.contains(&resp_info.status) {
                warn!("{}: 400 Response", resp_info.url);
            } else {
                let mut message = vec![format!("{}", url)];
                if show_status {
                    message.push(format!("{}", resp_info.status));
                }

                if show_title {
                    message.push(resp_info.title.replace("\n", ""));
                }
                println!("{}", message.join(delimiter));
                return Some(resp_info);
            }
        }
        Err(err) => {
            warn!("{}", err);
        }
    };

    return None;
}

#[derive(Serialize, Debug)]
struct ResponseInfo {
    pub url: String,
    pub status: u16,
    pub title: String,
}

async fn parse_response(resp: Response) -> ResponseInfo {
    let status = resp.status().as_u16();
    let url = format!("{}", resp.url());
    let title = get_resp_title(resp).await.unwrap_or("".to_string());
    return ResponseInfo { url, status, title };
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


fn save_json(filepath: String, responses: &Vec<ResponseInfo>) {
    let json_str = serde_json::to_string(responses).expect("Error parsing responses");


    match File::create(&filepath) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(json_str.as_bytes()) {
                error!("Error writing {}: {}", filepath, err);
            }
        }
        Err(err) => {
            error!("Error opening {}: {}", filepath, err);
        }
    }
}
