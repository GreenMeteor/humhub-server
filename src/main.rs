extern crate ssh2;
extern crate trust_dns_resolver;
extern crate serde;
extern crate serde_json;

use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::fs::File;

use ssh2::Session;
use trust_dns_resolver::{Resolver, config::ResolverConfig, system_conf::read_system_conf};
use serde::{Deserialize, Serialize};

// Define a struct to deserialize the JSON configuration
#[derive(Debug, Deserialize)]
struct Config {
    host: String,
    username: String,
    password: String,
    domain_name: String,
    server_ip: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    // Read JSON file and deserialize into Config struct
    let config_file = File::open("config.json")?;
    let config: Config = serde_json::from_reader(config_file)?;

    // Additional PHP extensions required by HumHub
    let php_extensions = vec![
        "php8.1-intl",
        "php8.1-bcmath",
        "php8.1-gmp",
        "php8.1-ldap",
    ];

    // Update DNS records to point to the server's IP address
    let resolver = match Resolver::new(ResolverConfig::default(), read_system_conf()) {
        Ok(resolver) => resolver,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
    };
    let mut response = match resolver.update_record(&config.domain_name, &config.server_ip) {
        Ok(response) => response,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
    };

    println!("DNS record updated successfully: {:?}", response);

    // Connect to the server via SSH
    let tcp = match TcpStream::connect(format!("{}:22", config.host)) {
        Ok(tcp) => tcp,
        Err(err) => return Err(err),
    };
    let mut sess = match Session::new() {
        Ok(sess) => sess,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
    };
    if let Err(err) = sess.handshake(&tcp) {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    // Authenticate with username and password
    if let Err(err) = sess.userauth_password(&config.username, &config.password) {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    // Install PHP 8.1 and required extensions
    let mut commands = vec![
        "sudo apt update",
        "sudo apt upgrade -y",
        "sudo apt install -y apache2",
        "sudo add-apt-repository -y ppa:ondrej/php",
        "sudo apt update",
        "sudo apt install -y php8.1 libapache2-mod-php8.1 php8.1-mysql php8.1-common php8.1-cli php8.1-curl php8.1-json php8.1-zip php8.1-gd php8.1-mbstring php8.1-xml",
        "sudo apt install -y mariadb-server",
        "sudo mysql_secure_installation",
        "sudo a2enmod php8.1",
        "sudo systemctl restart apache2",
    ];

    // Add installation commands for PHP extensions required by HumHub
    for extension in &php_extensions {
        commands.push(&format!("sudo apt install -y {}", extension));
    }

    // Execute commands remotely
    let mut channel = match sess.channel_session() {
        Ok(channel) => channel,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
    };
    for cmd in &commands {
        if let Err(err) = channel.exec(cmd) {
            return Err(io::Error::new(io::ErrorKind::Other, err));
        }
        let mut output = String::new();
        if let Err(err) = channel.read_to_string(&mut output) {
            return Err(io::Error::new(io::ErrorKind::Other, err));
        }
        println!("{}", output);
    }

    // Modify Apache virtual host configuration to handle requests for the domain
    let apache_config = format!(
        r#"
        <VirtualHost *:80>
            ServerName {}
            DocumentRoot /var/www/html/{}
            <Directory /var/www/html/{}>
                Options Indexes FollowSymLinks
                AllowOverride All
                Require all granted
            </Directory>
        </VirtualHost>
        "#,
        &config.domain_name, &config.domain_name, &config.domain_name
    );

    if let Err(err) = channel.exec(&format!("echo '{}' | sudo tee /etc/apache2/sites-available/{}.conf", apache_config, &config.domain_name)) {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }
    if let Err(err) = channel.exec(&format!("sudo a2ensite {}.conf", &config.domain_name)) {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }
    if let Err(err) = channel.exec("sudo systemctl reload apache2") {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    // Close the SSH session
    if let Err(err) = channel.send_eof() {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }
    if let Err(err) = channel.wait_close() {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    Ok(())
}
