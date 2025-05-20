use clap::Parser;
use log::LevelFilter;
use pog::consensus::ConsensusType;
use pog::network;
use pog::network::graph::TopologyType;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "wujian", about = "POG协议模拟")]
struct Args {
    /// 节点个数
    #[clap(short, long, default_value = "50")]
    node_num: u32,

    /// 每秒交易个数（泊松分布）
    #[clap(short, long, default_value = "10")]
    trans_num: u32,

    /// 共识算法类型
    #[arg(short, long, default_value_t = ConsensusType::POG)]
    consensus: ConsensusType,

    ///拓扑结构
    #[arg(long, default_value_t = TopologyType::BA)]
    topology: TopologyType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //args
    let args = Args::parse();

    //log setting
    init_logger()?;

    network::start_network(args.node_num, args.trans_num, args.consensus, args.topology).await;
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
