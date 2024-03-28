use std::io::{self, Read};
use std::fs::File;
use serde::Deserialize;
use trust_dns_resolver::{Resolver, config::ResolverConfig, config::ResolverOpts};
use std::net::TcpStream;

#[derive(Debug, Deserialize)]
struct Config {
    domain_name: String,
    host: String,
    username: String,
    password: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    // Read configuration from file
    let config_file = File::open("config.json")?;
    let config: Config = serde_json::from_reader(config_file)?;

    // Initialize DNS resolver
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    let response = resolver.lookup_ip(&config.domain_name)?;
    println!("DNS record updated successfully: {:?}", response);

    // Connect to SSH server
    let _tcp = TcpStream::connect(format!("{}:22", config.host))?;
    let mut sess = ssh2::Session::new()?;

    // Handshake with SSH server
    if let Err(err) = sess.handshake() {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    // Authenticate with SSH server using username and password
    if let Err(err) = sess.userauth_password(&config.username, &config.password) {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    // Define commands to install required software
    let commands = [
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
    let mut channel = sess.channel_session()?;
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

    // Modify Apache virtual host configuration
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

    if let Err(err) = channel.exec(&format!(
        "echo '{}' | sudo tee /etc/apache2/sites-available/{}.conf",
        apache_config, &config.domain_name
    )) {
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
