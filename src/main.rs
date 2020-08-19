///
///

// TODO: timeout to milliseconds
// TODO: extract title from html

mod args;
mod ports;
mod readin;

use futures::{stream, StreamExt};
use std::time::Duration;

use args::Arguments;
use ports::ProtoPort;

use log::{info, warn};
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
    let client = create_http_client(args.timeout);
    let invalid_codes = &args.invalid_codes;
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
                            println!("{}", url);
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

fn create_http_client(timeout: u64) -> reqwest::Client {
    let builder = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(timeout));

    return builder.build().unwrap();
}


/// Function to generate urls, from protocol,
/// host and port and returns them in a iterator.
fn gen_urls(proto_ports: Vec<ProtoPort>, targets: Vec<String>) -> impl Iterator<Item = String> {
    return UrlGenerator::new(proto_ports, Box::new(readin::read_inputs(targets)));
}

struct UrlGenerator {
    proto_ports: Vec<ProtoPort>,
    targets: Box<dyn Iterator<Item = String>>,
    current_target: Option<String>,
    current_proto_ports: Vec<ProtoPort>,
}

impl UrlGenerator {
    fn new(proto_ports: Vec<ProtoPort>, targets: Box<dyn Iterator<Item = String>>) -> Self {
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
