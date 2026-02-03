mod app;

use anyhow::Result;
use clap::Parser;
use std::fs::OpenOptions;
use std::io::{self, Write};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "toast")]
#[command(about = "The browser - Render web pages in your terminal", long_about = None)]
struct Args {
    /// URL to render
    url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create log file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("toastylog.log")?;

    // Initialize tracing with file output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::sync::Mutex::new(log_file))
        .with_ansi(false) // Disable ANSI colors in log file
        .init();

    let args = Args::parse();

    // Get URL from args or prompt user
    let url_input = if let Some(url) = args.url {
        url
    } else {
        print!("\"Toast\" - the browser. Enter a URL: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Validate URL
    let url = if url_input.starts_with("http://") || url_input.starts_with("https://") {
        url_input
    } else {
        format!("https://{}", url_input)
    };

    let app = app::App::new(url);
    app.run().await
}
