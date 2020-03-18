mod arguments;
mod printer;

use std::fs::File;
use std::io::{BufRead, BufReader};
use futures::{stream, StreamExt};
use std::time::Duration;

use arguments::Arguments;
use printer::Printer;

#[tokio::main]
async fn main() {
    let args = Arguments::parse_args();

    let hosts_file = File::open(args.hosts_filename())
        .expect("Error opening ip file");
    
    let mut protocols = Vec::new();

    if args.use_http() {
        protocols.push("http")
    }

    if args.use_https() {
        protocols.push("https")
    }

    let ports = args.ports();
    
    let hosts_reader = BufReader::new(hosts_file);

    let client = create_http_client(
        args.timeout()
    );
    let urls = generate_urls(&protocols, hosts_reader, ports);

    let requests_max = urls.len();
    let bodies = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                return client.get(&url).send().await;
            }
        })
        .buffer_unordered(args.threads());

    let invalid_400 = args.invalid_400();
    let printer = Printer::new(args.progress(), args.verbosity());
    let mut requests_count = 0;
    bodies
        .for_each(|b| {
            let printer = &printer;
            requests_count = requests_count + 1;
            async move {
                match b {
                    Ok(resp) => {
                        
                        if invalid_400 && resp.status().as_u16() == 400 {
                            printer.print_error(
                                &format!("{}: 400 Response", resp.url())
                            );
                        }else {
                            printer.print_url(resp.url());
                        }
                    }
                    Err(err) => {
                        printer.print_error(&err);
                    }
                }
                printer.print_progress(requests_count, requests_max);
            }
        })
        .await;

        printer.print_end();
}

fn create_http_client(
    timeout: u64
) -> reqwest::Client {
    let builder = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(timeout));

    return builder.build().unwrap();
}

fn generate_urls(
    protocols: &[&str],
    hosts_reader: BufReader<File>,
    ports: &Vec<u16>
) -> Vec<String> {
    let mut urls = Vec::new();

    for line in hosts_reader.lines() {
        let hostname = line.unwrap();
        for port in ports.iter() {
            for protocol in protocols.iter() {
                let url = format!("{}://{}:{}", protocol, hostname, port);
                urls.push(url);
            }
        }
    }
    return urls;
}