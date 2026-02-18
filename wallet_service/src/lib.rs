pub mod api;
pub mod domain;
pub mod infrastructure;
pub mod use_cases;

#[tokio::main]
async fn main() {
    println!("Wallet Service Starting...");
}
