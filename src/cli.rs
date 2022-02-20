use std::fs::{File, OpenOptions};
use std::fs::read_to_string;
use serde_json::json;
use std::io::Write;

use anyhow::Result;

use crate::stocks::Stock;

pub fn add_stock(symbol: String, simulate: Option<bool>) -> Result<()> {
    let simulate = match simulate {
        Some(v) => v,
        None => false
    };
    let data = match read_to_string("./storage/stocks.json") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to read prestored stocks data!: {}", e);
            let _ = crate::stocks::mk_storage_dir();
            let _ = File::create("./storage/stocks.json");
            eprintln!("created directory");
            std::process::exit(1);
        }
    }; 

    let new_stock = Stock::new(symbol, 100, simulate);

    let mut data: crate::stocks::StockWrapper = serde_json::from_str(&data)?;

    data.data.push(new_stock);

    let mut save = match OpenOptions::new().write(true).open("./storage/stocks.json") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to load! due to: {}", e);
            let _ = crate::stocks::mk_storage_dir();

            let _ = File::create("./storage/stocks.json");
            eprintln!("created storage/stocks.json");
            std::process::exit(1);
        }
    };

    let stringify = json!(data).to_string();

    save.set_len(stringify.as_bytes().len() as u64)?;
    save.write_all(stringify.as_bytes())?; 
    Ok(())
}
