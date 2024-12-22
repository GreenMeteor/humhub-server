use std::io::{self, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use serde::Deserialize;
use reqwest::blocking;
use zip;
use std::time::Duration;

// Define a struct to deserialize the JSON configuration
#[derive(Debug, Deserialize)]
struct Config {
    domain_name: String,
}

// Function to encapsulate download functionality
pub fn download() -> io::Result<()> {
    // Load configuration
    let config = load_config("config.json")?;

    // Download and extract HumHub
    let humhub_version = "1.16.2";
    let humhub_download_url = format!(
        "https://download.humhub.com/downloads/install/humhub-{}.zip",
        humhub_version
    );
    let humhub_zip_path = Path::new(&format!("humhub-{}.zip", humhub_version));
    let humhub_extract_dir = Path::new("/var/www/html");

    download_file(&humhub_download_url, humhub_zip_path)?;
    extract_zip(humhub_zip_path, humhub_extract_dir)?;

    println!(
        "HumHub version {} downloaded and extracted successfully to {}",
        humhub_version,
        humhub_extract_dir.display()
    );

    Ok(())
}

// Function to load configuration from a JSON file
fn load_config(path: &str) -> io::Result<Config> {
    let config_file = File::open(path)?;
    serde_json::from_reader(config_file).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// Function to download a file from a URL
fn download_file(url: &str, output_path: &Path) -> io::Result<()> {
    println!("Downloading file from: {}", url);

    let client = blocking::Client::builder()
        .timeout(Duration::from_secs(60))  // Add timeout for the request
        .danger_accept_invalid_certs(false)  // Make sure SSL validation is done (set to true for secure config)
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create HTTP client: {}", e)))?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to send request: {}", e)))?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download file: HTTP {}", response.status())
        ));
    }

    let mut file = File::create(output_path)?;
    io::copy(&mut response, &mut file)?;

    println!("File downloaded to: {}", output_path.display());
    Ok(())
}

// Function to extract a ZIP file to a target directory
fn extract_zip(zip_path: &Path, extract_dir: &Path) -> io::Result<()> {
    println!("Extracting ZIP file: {}", zip_path.display());

    let zip_file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read ZIP archive: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_dir.join(sanitize_path(&file.sanitized_name(), extract_dir)?);

        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("Extraction completed to: {}", extract_dir.display());
    Ok(())
}

// Function to sanitize file paths and ensure they are within the target directory
fn sanitize_path(path: &Path, base_dir: &Path) -> io::Result<PathBuf> {
    let sanitized = path
        .strip_prefix("/")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

    let full_path = base_dir.join(sanitized);
    if full_path.starts_with(base_dir) {
        Ok(full_path)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Path traversal detected"))
    }
}
