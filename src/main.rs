extern crate ssh2;

use std::net::TcpStream;
use std::io::{Read, Write};

use ssh2::Session;

fn main() {
    // SSH connection parameters
    let host = "your_server_ip";
    let username = "your_username";
    let password = "your_password";
    let port = 22;

    // Commands to be executed remotely
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
        "sudo apt install -y php8.1-intl php8.1-bcmath php8.1-gmp php8.1-ldap",
    ];

    // Connect to the server via SSH
    let tcp = TcpStream::connect(format!("{}:{}", host, port)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp).unwrap();
    sess.userauth_password(username, password).unwrap();

    // Execute commands remotely
    let mut channel = sess.channel_session().unwrap();
    for cmd in &commands {
        channel.exec(cmd).unwrap();
        let mut output = String::new();
        channel.read_to_string(&mut output).unwrap();
        println!("{}", output);
    }

    // Close the SSH session
    channel.send_eof().unwrap();
    channel.wait_close().unwrap();
}
