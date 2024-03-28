extern crate reqwest;
extern crate zip;
extern crate serde;
extern crate serde_json;

use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use serde::Deserialize;

// Define a struct to deserialize the JSON configuration
#[derive(Debug, Deserialize)]
struct Config {
    domain_name: String,
}

// Function to encapsulate download functionality
pub fn download() -> io::Result<()> {
    // Read JSON file and deserialize into Config struct
    let config_file = File::open("config.json")?;
    let config: Config = serde_json::from_reader(config_file)?;

    // URL to download HumHub
    let humhub_download_url = "https://download.humhub.com/downloads/install/humhub-1.15.4.zip";

    // File path to save the downloaded HumHub ZIP file
    let humhub_zip_path = "humhub-1.15.4.zip";

    // Directory to extract HumHub ZIP file (root directory)
    let humhub_extract_dir = "/var/www/html";

    // Initialize HTTP client
    let client = reqwest::blocking::Client::new();

    // Download HumHub
    let mut response = client.get(humhub_download_url).send()?;
    let mut zip_file = File::create(humhub_zip_path)?;
    io::copy(&mut response, &mut zip_file)?;

    // Extract HumHub ZIP file to the root directory
    let extract_dir = Path::new(humhub_extract_dir);
    let zip_file = File::open(humhub_zip_path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_dir.join(file.sanitized_name());

        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("HumHub downloaded and extracted successfully to {}", humhub_extract_dir);

    Ok(())
}
