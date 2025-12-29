use backend::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // Load .env file from parent directory (project root)
    let _ = dotenvy::from_filename("../.env");
    // Also try current directory as fallback
    let _ = dotenvy::dotenv();

    cli::main::<App, Migrator>().await
}
