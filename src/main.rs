use std::net::TcpStream;
use std::io::{self, Read};
use std::fs::File;

use ssh2::Session;
use trust_dns_resolver::{Resolver, config::ResolverConfig, system_conf::read_system_conf};
use serde::Deserialize;

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

    // Update DNS records to point to the server's IP address
    let (resolver_config, resolver_opts) = match read_system_conf() {
        Ok((config, opts)) => (config, opts),
        Err(err) => return Err(err),
    };
    let resolver = match Resolver::new(resolver_config, resolver_opts) {
        Ok(resolver) => resolver,
        Err(err) => return Err(err),
    };
    let response = match resolver.update_record(&config.domain_name, &config.server_ip) {
        Ok(response) => response,
        Err(err) => return Err(err),
    };

    println!("DNS record updated successfully: {:?}", response);

    // Connect to the server via SSH
    let tcp = TcpStream::connect(format!("{}:22", config.host))?;
    let mut sess = Session::new()?;
    sess.handshake(&tcp)?;

    // Authenticate with username and password
    sess.userauth_password(&config.username, &config.password)?;

    // Install PHP 8.1 and required extensions
    let mut channel = sess.channel_session()?;
    let commands = vec![
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

    // Execute commands remotely
    for cmd in &commands {
        channel.exec(cmd)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
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
        config.domain_name, config.domain_name, config.domain_name
    );

    channel.exec(&format!("echo '{}' | sudo tee /etc/apache2/sites-available/{}.conf", apache_config, config.domain_name))?;
    channel.exec(&format!("sudo a2ensite {}.conf", config.domain_name))?;
    channel.exec("sudo systemctl reload apache2")?;

    // Close the SSH session
    channel.send_eof()?;
    channel.wait_close()?;

    Ok(())
}
