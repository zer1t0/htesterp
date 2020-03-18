use clap::{App, Arg};

fn args() -> App<'static, 'static> {
    App::new("http_or_https")
    .version("0.1")
    .about("Check if http or https works for targets")
    .arg(
        Arg::with_name("hosts-file")
        .takes_value(true)
        .help("file with IP/hostname per line")
        .required(true)
    )
    .arg(
        Arg::with_name("ports")
        .short("p")
        .long("ports")
        .takes_value(true)
        .use_delimiter(true)
        .help("Ports to scan, separated by comma")
        .default_value("80,443")
    )
    .arg(
        Arg::with_name("threads")
        .short("t")
        .long("threads")
        .takes_value(true)
        .help("Number of threads")
        .default_value("10")
    )
    .arg(
        Arg::with_name("timeout")
        .long("timeout")
        .takes_value(true)
        .help("Timeout (seconds)")
        .default_value("5")
    )
    .arg(
        Arg::with_name("verbosity")
        .short("v")
        .multiple(true)
        .help("Verbosity")
    )
    .arg(
        Arg::with_name("invalid-400")
        .long("invalid-400")
        .help("Filter reponses wit status 400 as not valid")
    )
    .arg(
        Arg::with_name("progress")
        .long("progress")
        .short("P")
        .help("Show the quantity of requests performed")
    )
    .arg(
        Arg::with_name("no-http")
        .long("no-http")
        .help("Do not check the HTTP protocol")
    )
    .arg(
        Arg::with_name("no-https")
        .long("no-https")
        .help("Do not check the HTTPS protocol")
        .conflicts_with("no-http")
    )
}

pub struct Arguments {
    hosts_filename: String,
    ports: Vec<u16>,
    threads: usize,
    timeout: u64,
    verbosity: u64,
    invalid_400: bool,
    progress: bool,
    use_http: bool,
    use_https: bool
}

impl Arguments {

    pub fn parse_args() -> Self {
        let matches = args().get_matches();
        let hosts_filename = matches.value_of("hosts-file").unwrap().to_string();
        let threads = matches.value_of("threads").unwrap().parse().expect("Invalid threads value");
        let timeout = matches.value_of("timeout").unwrap().parse().expect("Invalid timeout value");

        let verbosity = matches.occurrences_of("verbosity");
        let progress = matches.is_present("progress");

        let mut ports: Vec<u16> = Vec::new();
        for port in matches.values_of("ports").unwrap() {
            ports.push(
                port.parse().expect(
                    format!("Invalid port number {}", port).as_str()
                )
            );
        }

        let invalid_400 = matches.is_present("invalid-400");

        return Self {
            hosts_filename,
            ports,
            threads,
            timeout,
            verbosity,
            invalid_400,
            progress,
            use_http: !matches.is_present("no-http"),
            use_https: !matches.is_present("no-https")
        };
    }

    pub fn hosts_filename(&self) -> &String {
        return &self.hosts_filename;
    }

    pub fn ports(&self) -> &Vec<u16> {
        return &self.ports;
    }

    pub fn threads(&self) -> usize {
        return self.threads;
    }

    pub fn timeout(&self) -> u64 {
        return self.timeout;
    }

    pub fn verbosity(&self) -> u64 {
        return self.verbosity;
    }

    pub fn invalid_400(&self) -> bool {
        return self.invalid_400;
    }

    pub fn progress(&self) -> bool {
        return self.progress;
    }

    pub fn use_http(&self) -> bool {
        return self.use_http;
    }

    pub fn use_https(&self) -> bool {
        return self.use_https;
    }

}