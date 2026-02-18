pub mod api;
pub mod domain;
pub mod infrastructure;
pub mod use_cases;
// pub mod jobs; // Uncomment when implemented

#[tokio::main]
async fn main() {
    println!("Transaction Service Starting...");
}
