use std::io::{self, Read};
use std::fs::File;
use serde::Deserialize;
use trust_dns_resolver::{Resolver, config::ResolverConfig, config::ResolverOpts};
use std::net::TcpStream;
use ssh2::Session;
use std::time::Duration;

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
    // Load configuration
    let config = load_config("config.json")?;
    // Initialize DNS resolver and resolve domain
    resolve_domain(&config.domain_name)?;

    // Establish SSH connection
    let mut sess = establish_ssh_connection(&config)?;

    // Install required software on the remote server
    execute_remote_commands(
        &mut sess,
        &[
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
        ],
    )?;

    // Configure Apache
    configure_apache(&mut sess, &config)?;

    println!("Setup completed successfully.");
    Ok(())
}

fn load_config(path: &str) -> io::Result<Config> {
    let config_file = File::open(path)?;
    serde_json::from_reader(config_file).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn resolve_domain(domain: &str) -> io::Result<()> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    let response = resolver.lookup_ip(domain)?;
    println!("Resolved DNS for {}: {:?}", domain, response);
    Ok(())
}

fn establish_ssh_connection(config: &Config) -> io::Result<Session> {
    let tcp = TcpStream::connect(format!("{}:22", config.host))?;
    tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut sess = Session::new().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Failed to create SSH session")
    })?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(&config.username, &config.password)?;

    if !sess.authenticated() {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Authentication failed"));
    }

    Ok(sess)
}

fn execute_remote_commands(sess: &mut Session, commands: &[&str]) -> io::Result<()> {
    for cmd in commands {
        println!("Executing: {}", cmd);
        let mut channel = sess.channel_session()?;
        channel.exec(cmd)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        println!("{}", output);
        channel.wait_close()?;
    }
    Ok(())
}

fn configure_apache(sess: &mut Session, config: &Config) -> io::Result<()> {
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

    let commands = [
        format!(
            "echo '{}' | sudo tee /etc/apache2/sites-available/{}.conf",
            apache_config, config.domain_name
        ),
        format!("sudo a2ensite {}.conf", config.domain_name),
        "sudo systemctl reload apache2".to_string(),
    ];

    execute_remote_commands(sess, &commands.iter().map(String::as_str).collect::<Vec<_>>())
}
