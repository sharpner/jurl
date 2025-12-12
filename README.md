# jurl - JavaScript-enabled curl

A curl-like command-line tool with built-in JavaScript rendering capabilities using headless Chrome.

## Features

- Full JavaScript rendering using headless Chrome
- curl-compatible command-line interface
- Support for modern JavaScript-heavy websites
- Screenshot capture capability
- Multiple output formats (HTML, text, JSON)
- Custom headers and user agent support

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/jurl`

## Usage

Basic usage:
```bash
jurl https://example.com
```

With verbose output:
```bash
jurl -v https://example.com
```

Save to file:
```bash
jurl -o output.html https://example.com
```

Get text content only:
```bash
jurl --format text https://example.com
```

Take a screenshot:
```bash
jurl --screenshot page.png https://example.com
```

Wait for specific element:
```bash
jurl --wait-for-selector "div.content" https://example.com
```

## Command-line Options

- `-X, --request <METHOD>` - HTTP method (default: GET)
- `-i, --include` - Include headers in output
- `-v, --verbose` - Verbose output
- `-L, --location` - Follow redirects
- `-o, --output <FILE>` - Save output to file
- `-H, --header <HEADER>` - Add custom header
- `-d, --data <DATA>` - POST data
- `-A, --user-agent <UA>` - Custom user agent
- `-s, --silent` - Silent mode (no headers)
- `--wait-for-selector <SELECTOR>` - Wait for CSS selector
- `--timeout <SECONDS>` - Timeout in seconds (default: 30)
- `--screenshot <FILE>` - Take screenshot instead of HTML
- `--format <FORMAT>` - Output format: html, text, json (default: html)

## Examples

Fetch JavaScript-rendered content from WoW database:
```bash
jurl "https://www.wowhead.com/classic/item=18252/pattern-core-armor-kit" --format text
```

## Requirements

- Chrome/Chromium browser (automatically downloaded on first run)
- macOS, Linux, or Windows

## License

MIT