use alpha_vantage::api::ApiClient;
use std::cell::RefCell;
use std::io::Write;
use std::fs::File;
use anyhow::Result;
use std::{time::{SystemTime, UNIX_EPOCH}, fs::{self, OpenOptions}};
use serde_derive::{Serialize, Deserialize};
use serde_json::json;

pub struct Manager {
    pub key: String,
    pub client: RefCell<ApiClient>,
    pub stocks: Vec<Stock>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StockWrapper {
    pub data: Vec<Stock>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Stock {
    pub symbol: String,
    pub update: u64,    // last stock update in unix seconds
    pub open: u32,      // last open price
    pub close: u32,     // last closing price
    pub initial: u32,   // intial money put in,
    pub amt: u32,       // current money
    pub starting: bool, // starting
    pub simulate: bool  
}

impl Stock {
    pub fn new(symbol: String, amt:u32, simulate: bool) -> Stock {
        let amt = match simulate {
            true => 100000,
            false => amt,
        };

        Stock {
            symbol,
            update: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            open: 0,
            close: 0,
            initial: 0,
            amt,
            starting: true,
            simulate
        }
    }

    pub async fn fetch(&mut self, start: u64, end: Option<u64>, client: &ApiClient) -> Stock {
        let end = match end {
            Some(v) => v,
            None => SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() 
        };

        let amt = match self.starting {
            true => self.initial,
            false => self.amt 
        };

        let stock_time = client
            .stock_time(alpha_vantage::stock_time::StockFunction::IntraDay, &self.symbol.clone())
            .interval(alpha_vantage::api::TimeSeriesInterval::FifteenMin)
            .output_size(alpha_vantage::api::OutputSize::Compact)
            .json()
            .await
            .unwrap();

        let entries = stock_time.entry();

        //let current = entries[0].clone();
        let latest = entries[entries.len() - 1].clone();

        let mut new_amt = (latest.open() as f32 / latest.close() as f32) * amt as f32;

        let mut sell = false;

        if new_amt < (amt as f32 * 0.9999) {
            sell = true;
        }

        if sell {
            new_amt = amt as f32;
        }

        println!("symbol: {} starting: {} new: {:?} INIT: {}", 
                 self.symbol, 
                 amt as f32 / 100.0, 
                 new_amt / 100.0, 
                 self.initial as f32 / 100.0
                 );
        println!("\tgain: {:?} time: {:?}", 
                 (new_amt - amt as f32) / 100.0, 
                 latest.time()
                 );
        println!("\tsell? {} GAP: {}", 
                 sell, 
                 (new_amt - self.initial as f32) / 100.0
                 );

        Stock {
            symbol: self.symbol.clone(),
            update: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            open: (latest.open() * 100.0) as u32,
            close: (latest.close() * 100.0) as u32,
            initial: self.initial,
            amt: new_amt as u32, // TODO CALC
            starting: false,
            simulate: self.simulate
        }

    }
}

impl Manager {
    pub fn new(key: &str) -> Manager {
        let client = alpha_vantage::set_api(key, reqwest::Client::new());
        Manager {
            key: key.to_string(),
            client: RefCell::new(client),
            stocks: Vec::new()
        }
    } 

    pub fn save(&self) -> Result<()> {
        let mut save = match OpenOptions::new().write(true).open("./storage/stocks.json") {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to load! due to: {}", e);
                let _ = mk_storage_dir();

                let _ = File::create("./storage/stocks.json");
                eprintln!("created storage/stocks.json");
                std::process::exit(1);
            }
        };

        let contents = json!(self.stocks).to_string();

        let contents = format!("{{\"data\":{}}}", contents);

        save.set_len(contents.as_bytes().len() as u64)?;
        save.write_all(contents.as_bytes())?;

        Ok(())
    }

    pub async fn update_stocks(&mut self, start: u64, end: Option<u64>) -> Result<()> {
        // switch to refcell
        let mut new = Vec::new();
        for i in self.stocks.clone() {
            new.push(i.clone().fetch(start, end, &*self.client.borrow_mut()).await);
        };

        self.stocks = new;
        Ok(())
    }

    pub fn load_stocks(&mut self) -> Result<()> {
        let data = match fs::read_to_string("./storage/stocks.json") {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to read prestored stocks data!: {}", e);
                let _ = mk_storage_dir();
                let _ = File::create("./storage/stocks.json");
                eprintln!("created directory");
                std::process::exit(1);
            }
        };
        let result: StockWrapper = serde_json::from_str(&data)?;

        self.stocks = result.data.clone();

        for i in result.data {
            println!("loaded data for: {}", i.symbol);
        }

        Ok(())
    }
}

pub fn mk_storage_dir() -> Result<()> {
    fs::create_dir("./storage")?;
    Ok(())
}
