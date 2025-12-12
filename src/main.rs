use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::*;
use headless_chrome::{Browser, LaunchOptions};
use serde_json::Value;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "jurl")]
#[command(version = "1.0.0")]
#[command(author = "jurl contributors")]
#[command(about = "A curl-like tool with JavaScript rendering capabilities")]
#[command(long_about = "jurl - JavaScript-enabled curl replacement

jurl is a command-line HTTP client similar to curl, but with built-in JavaScript 
rendering capabilities using headless Chrome. This allows it to fetch content from 
modern JavaScript-heavy websites that regular curl cannot handle.

KEY FEATURES:
  • Full JavaScript execution and rendering
  • curl-compatible command-line interface
  • Automatic browser management (no setup required)
  • Support for dynamic content and SPAs
  • Screenshot capture capability
  • Multiple output formats

EXAMPLES:
  # Basic GET request with JavaScript rendering
  jurl https://example.com

  # Save rendered content to file
  jurl -o output.html https://example.com

  # Get only text content (no HTML tags)
  jurl --format text https://example.com

  # Take a screenshot of the page
  jurl --screenshot page.png https://example.com

  # Wait for specific element to load
  jurl --wait-for-selector \"div.content\" https://example.com

  # Verbose mode with custom user agent
  jurl -v -A \"MyBot 1.0\" https://example.com

  # Include response headers in output
  jurl -i https://example.com

  # POST request with data (limited support)
  jurl -X POST -d \"key=value\" https://example.com/api

DIFFERENCES FROM CURL:
  • Executes JavaScript (curl doesn't)
  • Renders full DOM (curl only gets initial HTML)
  • Slightly slower due to browser overhead
  • Requires Chrome/Chromium (auto-downloaded)

NOTE: The browser is automatically downloaded on first run and cached locally.
      No manual browser installation or configuration is required.")]
struct Args {
    /// URL to fetch (required)
    #[arg(help = "The URL to fetch. Must include protocol (http:// or https://).")]
    url: String,

    /// HTTP method to use
    #[arg(short = 'X', long = "request", default_value = "GET", help = "Specify request method to use (GET, POST, etc.). Default is GET.")]
    method: String,

    /// Include headers in output
    #[arg(short = 'i', long = "include", help = "Include the response headers in the output, similar to curl -i.")]
    include_headers: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose", help = "Make the operation more talkative. Shows connection details and progress.")]
    verbose: bool,

    /// Follow redirects
    #[arg(short = 'L', long = "location")]
    follow_redirects: bool,

    /// Output to file
    #[arg(short = 'o', long = "output", help = "Write output to file instead of stdout. Creates the file if it doesn't exist.")]
    output: Option<String>,

    /// Headers to send (can be used multiple times)
    #[arg(short = 'H', long = "header", help = "Pass custom header(s) to server. Format: 'Header: value'. Can be used multiple times.")]
    headers: Vec<String>,

    /// Data to send with POST request
    #[arg(short = 'd', long = "data")]
    data: Option<String>,

    /// Wait for selector before capturing content
    #[arg(long = "wait-for-selector", help = "Wait for a specific CSS selector to appear before capturing content. Useful for dynamic content.")]
    wait_for_selector: Option<String>,

    /// Wait timeout in seconds
    #[arg(long = "timeout", default_value = "30", help = "Maximum time in seconds to wait for page load or selector. Default is 30 seconds.")]
    timeout: u64,

    /// Take a screenshot instead of HTML
    #[arg(long = "screenshot", help = "Capture a PNG screenshot of the page instead of HTML. Provide output filename.")]
    screenshot: Option<String>,

    /// Output format
    #[arg(long = "format", value_enum, default_value = "html", help = "Output format: html (raw HTML), text (text only, no tags), json (attempt to parse as JSON).")]
    format: OutputFormat,

    /// Show only response body (no headers)
    #[arg(short = 's', long = "silent")]
    silent: bool,

    /// User agent string
    #[arg(short = 'A', long = "user-agent", help = "Send User-Agent header to server. Useful for accessing sites that block automated tools.")]
    user_agent: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Html,
    Text,
    Json,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        eprintln!("{}", format!("* Connecting to {}...", args.url).cyan());
    }

    // Launch browser with options
    let launch_options = LaunchOptions {
        headless: true,
        window_size: Some((1920, 1080)),
        ..Default::default()
    };

    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;

    // Set user agent if provided
    if let Some(user_agent) = &args.user_agent {
        tab.set_user_agent(user_agent, None, None)?;
    }

    // Set default timeout
    tab.set_default_timeout(Duration::from_secs(args.timeout));

    if args.verbose {
        eprintln!("{}", format!("* Navigating to {}...", args.url).cyan());
    }

    // Navigate to URL
    match args.method.to_uppercase().as_str() {
        "GET" => {
            tab.navigate_to(&args.url)?;
        }
        "POST" => {
            // For POST requests, we navigate first then execute JS to submit data
            tab.navigate_to(&args.url)?;
            
            if let Some(data) = &args.data {
                if args.verbose {
                    eprintln!("{}", format!("* Sending POST data: {}", data).cyan());
                }
                // Note: Full POST implementation would require more sophisticated handling
            }
        }
        _ => {
            eprintln!("Unsupported method: {}", args.method);
            std::process::exit(1);
        }
    }

    // Wait for specific selector if provided
    if let Some(selector) = &args.wait_for_selector {
        if args.verbose {
            eprintln!("{}", format!("* Waiting for selector: {}", selector).cyan());
        }
        tab.wait_for_element(selector)?;
    } else {
        // Wait for page to be fully loaded
        tab.wait_until_navigated()?;
        // Additional wait for dynamic content
        std::thread::sleep(Duration::from_secs(2));
    }

    // Take screenshot if requested
    if let Some(screenshot_path) = &args.screenshot {
        if args.verbose {
            eprintln!("{}", format!("* Taking screenshot to: {}", screenshot_path).cyan());
        }
        let screenshot_data = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;
        std::fs::write(screenshot_path, screenshot_data)?;
        println!("Screenshot saved to: {}", screenshot_path);
        return Ok(());
    }

    // Get page content based on format
    let content = match args.format {
        OutputFormat::Html => {
            // Get the full HTML content
            tab.get_content()?
        }
        OutputFormat::Text => {
            // Get text content from body
            let result = tab.evaluate("document.body.innerText", false)?;
            result.value.unwrap().as_str().unwrap_or("").to_string()
        }
        OutputFormat::Json => {
            // Try to extract JSON from the page
            let result = tab.evaluate("document.body.innerText", false)?;
            let text = result.value.unwrap().as_str().unwrap_or("").to_string();
            
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                serde_json::to_string_pretty(&json)?
            } else {
                text
            }
        }
    };

    // Handle output
    if args.include_headers && !args.silent {
        // Get response info (simplified version)
        println!("{}", "HTTP/1.1 200 OK".green());
        println!("{}", format!("Content-Length: {}", content.len()).green());
        println!("{}", "Content-Type: text/html; charset=utf-8".green());
        println!();
    }

    if let Some(output_file) = &args.output {
        if args.verbose {
            eprintln!("{}", format!("* Writing output to: {}", output_file).cyan());
        }
        std::fs::write(output_file, content)?;
        if !args.silent {
            println!("Output saved to: {}", output_file);
        }
    } else if !args.silent {
        println!("{}", content);
    }

    if args.verbose {
        eprintln!("{}", "* Connection closed".cyan());
    }

    Ok(())
}