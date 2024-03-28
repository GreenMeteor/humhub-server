extern crate ssh2;
extern crate trust_dns_resolver;

use std::net::TcpStream;
use std::io::{Read, Write};
use std::env;

use ssh2::Session;
use trust_dns_resolver::{Resolver, config::ResolverConfig, system_conf::read_system_conf};

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        println!("Usage: {} <host> <username> <password> <domain_name> <server_ip>", args[0]);
        return;
    }
    let host = &args[1];
    let username = &args[2];
    let password = &args[3];
    let domain_name = &args[4];
    let server_ip = &args[5];

    // Additional PHP extensions required by HumHub
    let php_extensions = vec![
        "php8.1-intl",
        "php8.1-bcmath",
        "php8.1-gmp",
        "php8.1-ldap",
    ];

    // Update DNS records to point to the server's IP address
    let resolver = Resolver::new(ResolverConfig::default(), read_system_conf().unwrap()).unwrap();
    let mut response = resolver.update_record(domain_name, server_ip).unwrap();

    println!("DNS record updated successfully: {:?}", response);

    // Connect to the server via SSH
    let tcp = TcpStream::connect(format!("{}:22", host)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp).unwrap();
    sess.userauth_password(username, password).unwrap();

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
    let mut channel = sess.channel_session().unwrap();
    for cmd in &commands {
        channel.exec(cmd).unwrap();
        let mut output = String::new();
        channel.read_to_string(&mut output).unwrap();
        println!("{}", output);
    }

    // Modify Apache virtual host configuration to handle requests for the domain
    let apache_config = format!(
        r#"
        <VirtualHost *:80>
            ServerName {}
            DocumentRoot /var/www/{}
            <Directory /var/www/{}>
                Options Indexes FollowSymLinks
                AllowOverride All
                Require all granted
            </Directory>
        </VirtualHost>
        "#,
        domain_name, domain_name, domain_name
    );

    channel.exec(&format!("echo '{}' | sudo tee /etc/apache2/sites-available/{}.conf", apache_config, domain_name)).unwrap();
    channel.exec(&format!("sudo a2ensite {}.conf", domain_name)).unwrap();
    channel.exec("sudo systemctl reload apache2").unwrap();

    // Close the SSH session
    channel.send_eof().unwrap();
    channel.wait_close().unwrap();
}
