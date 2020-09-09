use crate::ports::ProtoPort;
use clap::{App, Arg, ArgMatches};

fn args() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("host")
                .takes_value(true)
                .help("Specify a host (IP or domain) or a file with hosts")
                .multiple(true)
                .use_delimiter(true),
        )
        .arg(
            Arg::with_name("ports")
                .short("p")
                .long("ports")
                .takes_value(true)
                .use_delimiter(true)
                .help("Ports to scan, separated by comma")
                .default_value("http:80,https:443")
                .validator(is_port),
        )
        .arg(
            Arg::with_name("title")
                .long("title")
                .short("t")
                .help("Show page title if any"),
        )
        .arg(
            Arg::with_name("status")
                .long("status")
                .short("s")
                .help("Show response status code"),
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("delimiter")
                .takes_value(true)
                .help("Fields delimiter")
                .default_value(" "),
        )
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .takes_value(true)
                .help("Number of workers")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .short("T")
                .takes_value(true)
                .help("Timeout (milliseconds)")
                .default_value("5000"),
        )
        .arg(
            Arg::with_name("redirect")
                .long("location")
                .short("L")
                .help("Follow redirects"),
        )
        .arg(
            Arg::with_name("json-file")
                .short("j")
                .long("json-file")
                .takes_value(true)
                .help("File to save data in json format")
                .default_value(" "),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Verbosity"),
        )
        .arg(
            Arg::with_name("invalid-400")
                .long("invalid-400")
                .help("Filter reponses wit status 400 as not valid"),
        )
}

pub fn is_port(v: String) -> Result<(), String> {
    if let Ok(_) = v.parse::<u16>() {
        return Ok(());
    }
    let parts: Vec<&str> = v.split(":").collect();

    if parts.len() != 2 {
        return Err(format!(
            "Invalid port '{}'. Format is [http[s]:]<0-65535>",
            v
        ));
    }

    if parts[0] != "http" && parts[0] != "https" {
        return Err(format!(
            "Invalid port '{}'. Format is [http[s]:]<0-65535>",
            v
        ));
    }

    if let Ok(_) = parts[1].parse::<u16>() {
        return Ok(());
    }
    return Err(format!(
        "Invalid port '{}'. Format is [http[s]:]<0-65535>",
        v
    ));
}

pub struct Arguments {
    pub targets: Vec<String>,
    pub proto_ports: Vec<ProtoPort>,
    pub workers: usize,
    pub timeout: u64,
    pub verbosity: usize,
    pub invalid_codes: Vec<u16>,
    pub show_title: bool,
    pub show_status: bool,
    pub delimiter: String,
    pub follow_redirect: bool,
    pub json_file: Option<String>,
}

impl Arguments {
    pub fn parse_args() -> Self {
        let matches = args().get_matches();
        let workers = matches
            .value_of("workers")
            .unwrap()
            .parse()
            .expect("Invalid threads value");
        let timeout = matches
            .value_of("timeout")
            .unwrap()
            .parse()
            .expect("Invalid timeout value");

        let verbosity = matches.occurrences_of("verbosity") as usize;

        return Self {
            targets: Self::parse_targets(&matches),
            proto_ports: Self::parse_ports(&matches),
            workers,
            timeout,
            verbosity,
            invalid_codes: Self::parse_invalid_codes(&matches),
            show_title: matches.is_present("title"),
            show_status: matches.is_present("status"),
            delimiter: matches.value_of("delimiter").unwrap().to_string(),
            follow_redirect: matches.is_present("redirect"),
            json_file: Self::parse_json_file(&matches),
        };
    }

    fn parse_invalid_codes(matches: &ArgMatches) -> Vec<u16> {
        if matches.is_present("invalid-400") {
            return vec![400];
        }
        return Vec::new();
    }

    fn parse_json_file(matches: &ArgMatches) -> Option<String> {
        return Some(matches.value_of("json-file")?.to_string())
    }

    fn parse_targets(matches: &ArgMatches) -> Vec<String> {
        if let Some(hosts) = matches.values_of("host") {
            return hosts.map(|s| s.to_string()).collect();
        }
        return Vec::new();
    }

    fn parse_ports(matches: &ArgMatches) -> Vec<ProtoPort> {
        let mut pps: Vec<ProtoPort> = Vec::new();
        for pp_str in matches.values_of("ports").unwrap() {
            let parts: Vec<&str> = pp_str.split(":").collect();

            let port: u16;
            let protocols: Vec<String>;

            match parts.len() {
                1 => {
                    protocols = vec!["http".to_string(), "https".to_string()];
                    port = parts[0].parse().unwrap();
                }
                2 => {
                    protocols = vec![parts[0].to_string()];
                    port = parts[1].parse().unwrap();
                }
                _ => unreachable!("Too much port parts"),
            }

            for proto in protocols.into_iter() {
                pps.push(ProtoPort::new(proto, port));
            }
        }

        return pps;
    }
}
