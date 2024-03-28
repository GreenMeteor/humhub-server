extern crate reqwest;
extern crate webdriver_client;
extern crate zip;

use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::process::Command;

use reqwest::blocking::Client;
use webdriver_client::{Browser, DesiredCapabilities, Session};

fn main() {
    // Domain name where HumHub will be installed
    let domain_name = std::env::args().nth(1).expect("Domain name argument missing");

    // URL to download HumHub
    let humhub_download_url = "https://download.humhub.com/downloads/install/humhub-1.15.4.zip";

    // File path to save the downloaded HumHub ZIP file
    let humhub_zip_path = "humhub-1.15.4.zip";

    // Directory to extract HumHub ZIP file (root directory)
    let humhub_extract_dir = "/var/www/html";

    // Initialize HTTP client
    let client = Client::new();

    // Download HumHub
    let mut response = client.get(humhub_download_url).send().unwrap();
    let mut zip_file = File::create(humhub_zip_path).unwrap();
    copy(&mut response.bytes().unwrap().as_ref(), &mut zip_file).unwrap();

    // Extract HumHub ZIP file to the root directory
    let extract_dir = Path::new(humhub_extract_dir);
    let zip_file = File::open(humhub_zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = extract_dir.join(file.sanitized_name());

        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            let mut outfile = File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    // Execute main.rs as SSH script
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--bin")
        .arg("main")
        .arg(domain_name)
        .output()
        .expect("Failed to execute SSH script");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
