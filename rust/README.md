# AI-Server-Discover

AI-LAN Service Discovery System - A tool for discovering AI services on local networks.

## Features
- Automatic discovery of AI services on local networks
- Service registration and management
- Network scanning for AI agents
- Configuration management

## Installation

### From GitHub Releases
1. Download the latest release for your platform from the [Releases page](https://github.com/yourusername/AI-Server-Discover/releases)
2. Extract the archive
3. Run the executable

### From Source
```bash
# Clone the repository
git clone https://github.com/yourusername/AI-Server-Discover.git
cd AI-Server-Discover/rust

# Build the project
cargo build --release

# Run the executable
./target/release/aiecho
```

## Usage

```bash
# Start the service discovery
aiecho

# With custom configuration
aiecho --config path/to/config.json
```

## Configuration

The service uses a JSON configuration file with the following structure:

```json
{
  "port": 5000,
  "scan_interval": 30,
  "services": []
}
```

## License

MIT
