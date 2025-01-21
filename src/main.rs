use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

mod blockchain;
mod network;
mod tools;
mod wallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //log setting
    init_logger()?;

    network::start_network(10, 50).await;
    Ok(())
}

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .set_time_format_str("%Y-%m-%d %H:%M:%S")
        .build();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            config,
            File::create("output.log").unwrap(),
        ),
    ])
    .unwrap();
    Ok(())
}
