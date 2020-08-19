# HTesTerP

Check if http and https protocols are available for several hosts

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
