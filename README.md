# thressh

Welcome to thressh, maintained by dangardner.

## Description

Scan hosts for a known SSH private key

## Getting Started

```bash
git clone https://github.com/dangardner/thressh.git
cd thressh
cargo build
```

## Usage

```bash
thressh [OPTIONS] --keyfile <KEYFILE> --targets <TARGETS>... --usernames <USERNAMES>...

Options:
      --keyfile <KEYFILE>            Name of file containing SSH private key
      --targets <TARGETS>...         Comma separated list of target hosts (hostnames or IP addresses)
      --targetfile <TARGETFILE>      File containing target hosts (hostnames or IP addresses)
      --usernames <USERNAMES>...     Comma separated list of usernames to use for authentication
      --usernamefile <USERNAMEFILE>  File containing usernames to use for authentication
      --tasks <TASKS>                Number of tasks to run concurrently [default: 20]
      --maxconns <MAXCONNS>          Maximum number of concurrent connections per host (may block other tasks) [default: 1]
      --timeout <TIMEOUT>            TCP connection timeout, in milliseconds [default: 2000]
  -h, --help                         Print help
  -V, --version                      Print version
```

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.
