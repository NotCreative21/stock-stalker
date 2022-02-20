use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use std::thread::sleep;
use anyhow::Result;
use std::env;

mod stocks;
use stocks::*;
mod config;
use config::*;
mod cli;
use cli::*;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() > 0 {
        match args[0].as_str() {
            "add" => {
                if args.len() > 2 {
                    match args[2].as_str() {
                        "true" => add_stock(args[1].clone(), Some(true))?,
                        "false" => add_stock(args[1].clone(), Some(false))?,
                        _ => add_stock(args[1].clone(), Some(false))?,
                    }; 
                    std::process::exit(0);
                }
                add_stock(args[1].clone(), Some(false))?;
                std::process::exit(0);
            },
            _ => {
                println!("invalid argument");
                std::process::exit(1);
            }
        };
    }

    let startup = Instant::now();
    let config = Config::load();
    let mut api_key = Manager::new(&config.key);
    api_key.load_stocks()?;
    println!("loaded data in: {:#?}", startup.elapsed());
    loop {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        println!("-------------------------------------");


        api_key.update_stocks(now - config.wait, Some(now)).await?;
        api_key.save()?;


        sleep(Duration::from_secs(config.wait));
    }
}
