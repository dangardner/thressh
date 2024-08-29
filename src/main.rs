extern crate futures;
extern crate log;
extern crate tokio;
extern crate tokio_stream;

use anyhow::{Context, Result};
use async_ssh2_lite::{AsyncSession, TokioTcpStream};
use clap::Parser;
use futures::StreamExt;
use itertools::iproduct;
use patharg::InputArg;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Scan hosts for a known SSH private key
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of file containing SSH private key
    #[arg(long)]
    keyfile: String,

    /// Comma separated list of target hosts (hostnames or IP addresses)
    #[arg(long, required = true, value_delimiter = ',', num_args = 1..)]
    targets: Vec<String>,

    /// File containing target hosts (hostnames or IP addresses)
    #[arg(long, conflicts_with = "targets")]
    targetfile: Option<InputArg>,

    /// Comma separated list of usernames to use for authentication
    #[arg(long, required = true, value_delimiter = ',', num_args = 1..)]
    usernames: Vec<String>,

    /// File containing usernames to use for authentication
    #[arg(long, conflicts_with = "usernames")]
    usernamefile: Option<InputArg>,

    /// Number of tasks to run concurrently
    #[arg(long, default_value_t = 20)]
    tasks: usize,

    /// Maximum number of concurrent connections per host (may block other tasks)
    #[arg(long, default_value_t = 1)]
    maxconns: u32,

    /// TCP connection timeout, in milliseconds
    #[arg(long, default_value_t = 2000)]
    timeout: u64,
}

fn read_lines(filearg: &InputArg) -> anyhow::Result<Vec<String>> {
    let file = filearg.open()?;
    let reader = BufReader::new(file);
    Ok(reader.lines().map_while(Result::ok).collect())
}

async fn ssh_authenticate(
    lock: Arc<RwLock<String>>,
    username: String,
    key: String,
    timeout: u64,
) -> anyhow::Result<(String, String)> {
    let target = lock.read().await;
    info!("Scanning {username}@{target}");

    let stream = tokio::time::timeout(
        std::time::Duration::from_millis(timeout),
        TokioTcpStream::connect((target.clone(), 22)),
    )
    .await?
    .context(format!("Failed to connect as {username}@{target}"))?;

    let mut session = AsyncSession::new(stream, None)?;
    session.handshake().await?;
    debug!("Authenticating as {username}@{target}");
    session
        .userauth_pubkey_memory(username.as_ref(), None, key.as_ref(), None)
        .await
        .context(format!("Failed to authenticate as {username}@{target}"))
        .and(Ok((target.to_string(), username)))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let key = String::from_utf8(std::fs::read(args.keyfile).unwrap()).unwrap();

    let targets = match args.targetfile {
        Some(file) => read_lines(&file).unwrap(),
        None => args.targets,
    }
    .into_iter()
    .map(|target| Arc::new(RwLock::with_max_readers(target, args.maxconns)));

    let usernames: Vec<String> = match args.usernamefile {
        Some(file) => read_lines(&file).unwrap(),
        None => args.usernames,
    };

    let mut stream = tokio_stream::iter(iproduct!(targets, usernames))
        .map(|t| ssh_authenticate(t.0, t.1, key.clone(), args.timeout))
        .buffer_unordered(args.tasks);

    while let Some(response) = stream.next().await {
        match response {
            Ok(response) => {
                debug!(
                    "Successfully authenticated as {}@{}",
                    response.1, response.0
                );
                println!("{}@{}", response.1, response.0);
            }
            Err(e) => {
                debug!("error: {e:?}");
            }
        }
    }
}
