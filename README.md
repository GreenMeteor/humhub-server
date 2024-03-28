# HumHub Server

| Status | Badges |
|-------|----------|
| CI Build Status | [![Rust CI](https://github.com/GreenMeteor/humhub-server/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/GreenMeteor/humhub-server/actions/workflows/rust.yml) |
| GitHub Issues | ![GitHub Issues](https://img.shields.io/github/issues/greenmeteor/humhub-server.svg) |
| GitHub Pull Requests | ![GitHub Pull Requests](https://img.shields.io/github/issues-pr/greenmeteor/humhub-server.svg) |
| License | ![License](https://img.shields.io/badge/License-AGPL-license?logo=github) |

HumHub Server is a Rust application for automating the deployment and setup of a HumHub instance on a remote server. It leverages SSH to execute commands remotely and configure the server according to the provided settings.

## Features

- **Automated Setup**: Quickly deploy a HumHub instance on a remote server without manual intervention.
- **Secure Configuration**: Utilizes SSH for secure communication and authentication.
- **Flexible Configuration**: Customize the deployment settings through a JSON configuration file.
- **Error Handling**: Robust error handling to gracefully handle failures during deployment.

## Requirements

- Rust Programming Language
- SSH access to the target server
- `config.json` file containing deployment settings (See example below)

## Usage

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/humhub-server.git
   ```

2. Customize the `config.json` file with your deployment settings. Example:

   ```json
   {
       "domain_name": "example.com",
       "host": "123.456.789.0",
       "username": "admin",
       "password": "yourpassword"
   }
   ```

3. Build and run the application:

   ```bash
   cargo run
   ```

## Security Considerations

- Ensure that the `config.json` file containing sensitive information (e.g., passwords) is stored securely and not shared publicly.
- Use strong passwords and SSH key-based authentication for secure communication with the server.

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or create a pull request.
