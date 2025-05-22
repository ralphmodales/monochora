use anyhow::{Context, Result, anyhow};
use reqwest;
use std::io::Write;
use std::path::{PathBuf};
use tempfile::NamedTempFile;
use url::Url;

pub async fn download_gif_from_url(url: &str) -> Result<PathBuf> {
    let parsed_url = Url::parse(url)
        .context("Invalid URL provided")?;
    
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(anyhow!("Only HTTP and HTTPS URLs are supported"));
    }
    
    println!("Downloading GIF from: {}", url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("monochora-gif-converter/1.0")
        .build()
        .context("Failed to create HTTP client")?;
    
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to send HTTP request")?;
    
    if !response.status().is_success() {
        return Err(anyhow!("HTTP request failed with status: {}", response.status()));
    }
    
    if let Some(content_type) = response.headers().get("content-type") {
        let content_type_str = content_type.to_str().unwrap_or("");
        if !content_type_str.starts_with("image/") {
            println!("Warning: Content-Type is '{}', which may not be an image", content_type_str);
        }
    }
    
    let total_size = response.content_length();
    if let Some(size) = total_size {
        println!("Downloading {} bytes...", size);
    }
    
    let file_extension = get_file_extension_from_url(&parsed_url)
        .unwrap_or_else(|| "gif".to_string());
    
    let mut temp_file = NamedTempFile::with_suffix(&format!(".{}", file_extension))
        .context("Failed to create temporary file")?;
    
    let bytes = response.bytes().await
        .context("Failed to download file content")?;
    
    temp_file.write_all(&bytes)
        .context("Failed to write downloaded content to temporary file")?;
    
    let temp_path = temp_file.into_temp_path();
    let final_path = temp_path.keep()
        .context("Failed to persist temporary file")?;
    
    println!("Downloaded successfully to temporary file: {}", final_path.display());
    
    Ok(final_path)
}

fn get_file_extension_from_url(url: &Url) -> Option<String> {
    if let Some(path_segments) = url.path_segments() {
        if let Some(last_segment) = path_segments.last() {
            if let Some(dot_pos) = last_segment.rfind('.') {
                let extension = &last_segment[dot_pos + 1..];
                match extension.to_lowercase().as_str() {
                    "gif" | "png" | "jpg" | "jpeg" | "webp" => {
                        return Some(extension.to_lowercase());
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

pub fn is_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

pub async fn get_input_path(input: &str) -> Result<PathBuf> {
    if is_url(input) {
        download_gif_from_url(input).await
    } else {
        Ok(PathBuf::from(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com/image.gif"));
        assert!(is_url("http://example.com/image.gif"));
        assert!(!is_url("/path/to/local/file.gif"));
        assert!(!is_url("file.gif"));
    }
    
    #[test]
    fn test_get_file_extension_from_url() {
        let url = Url::parse("https://example.com/path/image.gif").unwrap();
        assert_eq!(get_file_extension_from_url(&url), Some("gif".to_string()));
        
        let url = Url::parse("https://example.com/path/image.PNG").unwrap();
        assert_eq!(get_file_extension_from_url(&url), Some("png".to_string()));
        
        let url = Url::parse("https://example.com/path/file").unwrap();
        assert_eq!(get_file_extension_from_url(&url), None);
    }
}
