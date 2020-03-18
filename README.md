# HTesTerP

Check if http and https protocols are available for several hosts

## Example

```shell
$ echo "wikipedia.org
> crates.io
> fsf.org" > /tmp/http_hosts

$ htesterp /tmp/http_hosts
http://wikipedia.org/
http://crates.io/
http://crates.io:443/
https://wikipedia.org/
http://fsf.org/
https://crates.io/
https://fsf.org/

$ # discard the 400 responses, which usually indicates SSL error when requested HTTP
$ htesterp /tmp/http_hosts --invalid-400
http://crates.io/
http://wikipedia.org/
https://wikipedia.org/
http://fsf.org/
https://crates.io/
https://fsf.org/

```

## Usage

```
$ htesterp -h
htesterp 0.1.1
Eloy Perez <zer1t0ps@protonmail.com>
Tool to check if HTTP or HTTPS are available for hosts

USAGE:
    htesterp [FLAGS] [OPTIONS] <hosts-file>

FLAGS:
    -h, --help           Prints help information
        --invalid-400    Filter reponses wit status 400 as not valid
        --no-http        Do not check the HTTP protocol
        --no-https       Do not check the HTTPS protocol
    -P, --progress       Show the quantity of requests performed
    -V, --version        Prints version information
    -v                   Verbosity

OPTIONS:
    -p, --ports <ports>        Ports to scan, separated by comma [default: 80,443]
    -t, --threads <threads>    Number of threads [default: 10]
        --timeout <timeout>    Timeout (seconds) [default: 5]

ARGS:
    <hosts-file>    file with IP/hostname per line
```

## Installation

From crates.io (cargo):
```shell
cargo install htesterp
htesterp -h
```

From source:
```
git clone https://gitlab.com/Zer1t0/htesterp.git
cd ./htesterp
cargo build --release
./target/release/htesterp -h
```
