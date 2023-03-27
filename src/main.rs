mod discovery;
mod chain;

use std::io::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::thread;
use std::thread::park;
use std::time::Duration;
use async_std::task::block_on;
use env_logger::Env;
use clap::Parser;



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, )]
    port: Option<String>,

    #[arg(short, long)]
    nodes: Option<Vec<String>>,
}


#[async_std::main]
async fn main() {
    let args = Args::parse();
    println!(" ▄▄▄       ██▓███  ▓█████  ██▀███   ██▓ ▒█████   ███▄    █
▒████▄    ▓██░  ██▒▓█   ▀ ▓██ ▒ ██▒▓██▒▒██▒  ██▒ ██ ▀█   █
▒██  ▀█▄  ▓██░ ██▓▒▒███   ▓██ ░▄█ ▒▒██▒▒██░  ██▒▓██  ▀█ ██▒
░██▄▄▄▄██ ▒██▄█▓▒ ▒▒▓█  ▄ ▒██▀▀█▄  ░██░▒██   ██░▓██▒  ▐▌██▒
 ▓█   ▓██▒▒██▒ ░  ░░▒████▒░██▓ ▒██▒░██░░ ████▓▒░▒██░   ▓██░
 ▒▒   ▓▒█░▒▓▒░ ░  ░░░ ▒░ ░░ ▒▓ ░▒▓░░▓  ░ ▒░▒░▒░ ░ ▒░   ▒ ▒
  ▒   ▒▒ ░░▒ ░      ░ ░  ░  ░▒ ░ ▒░ ▒ ░  ░ ▒ ▒░ ░ ░░   ░ ▒░
  ░   ▒   ░░          ░     ░░   ░  ▒ ░░ ░ ░ ▒     ░   ░ ░
      ░  ░            ░  ░   ░      ░      ░ ░           ░
                                                           ");


    // Initialize the logger
    let env = Env::default().filter_or("MY_APP_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let handle = discovery::swarm::start(args);



    handle.join();
    // thread::sleep(Duration::from_millis(10000))
}
