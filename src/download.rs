extern crate reqwest;
extern crate webdriver_client;
extern crate zip;
extern crate serde;
extern crate serde_json;

use std::{
    fs::{self, File},
    io::{self, copy},
    path::Path,
};

use reqwest::blocking::Client;
use serde::Deserialize;
use webdriver_client::{Browser, DesiredCapabilities, Session};
use zip::ZipArchive;

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

    // Extract domain name from the configuration
    let domain_name = &config.domain_name;

    // URL to download HumHub
    let humhub_download_url = "https://download.humhub.com/downloads/install/humhub-1.15.4.zip";

    // File path to save the downloaded HumHub ZIP file
    let humhub_zip_path = "humhub-1.15.4.zip";

    // Directory to extract HumHub ZIP file (root directory)
    let humhub_extract_dir = "/var/www/html";

    // Initialize HTTP client
    let client = Client::new();

    // Download HumHub
    let mut response = client.get(humhub_download_url).send()?;
    let mut zip_file = File::create(humhub_zip_path)?;
    copy(&mut response.bytes()?.as_ref(), &mut zip_file)?;

    // Extract HumHub ZIP file to the root directory
    let extract_dir = Path::new(humhub_extract_dir);
    let zip_file = File::open(humhub_zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_dir.join(file.sanitized_name());

        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
        }
    }

    println!("HumHub downloaded and extracted successfully to {}", humhub_extract_dir);

    // Set up WebDriver session
    let mut session = Session::new(Browser::Chrome, DesiredCapabilities::chrome())?;
    session.open()?;

    // Navigate to the domain name
    session.navigate(&format!("http://{}", domain_name))?;

    // Close the WebDriver session
    session.close()?;
    
    Ok(())
}
