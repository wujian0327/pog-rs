mod blockchain;
mod network;
mod tools;
mod wallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    Ok(())
}
