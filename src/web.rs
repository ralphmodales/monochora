use crate::{MonochoraError, Result};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use url::Url;
use tracing::{debug, info, warn};

pub async fn download_gif_from_url(url: &str) -> Result<PathBuf> {
    let parsed_url = Url::parse(url)
        .map_err(|e| MonochoraError::UrlParse(e))?;
    
    match parsed_url.scheme() {
        "http" | "https" => {},
        scheme => return Err(MonochoraError::InvalidUrlScheme { 
            scheme: scheme.to_string() 
        }),
    }
    
    info!("Downloading GIF from: {}", url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("monochora-gif-converter/1.0")
        .build()
        .map_err(|e| MonochoraError::Http(e))?;
    
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| MonochoraError::Http(e))?;
    
    if !response.status().is_success() {
        return Err(MonochoraError::Io(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP request failed with status: {}", response.status())
            )
        ));
    }
    
     if let Some(content_type) = response.headers().get("content-type") {
        match content_type.to_str() {
            Ok(content_type_str) => {
                if !content_type_str.starts_with("image/") {
                    warn!("Content-Type is '{}', which may not be an image", content_type_str);
                }
            }
            Err(e) => {
                warn!("Failed to parse Content-Type header: {}", e);
            }
        }
    }
    
     if let Some(size) = response.content_length() {
        info!("Downloading {} bytes...", size);
        
         if size > 100_000_000 {
            warn!("File size is very large: {} bytes", size);
        }
    }
    
    let file_extension = get_file_extension_from_url(&parsed_url)
        .unwrap_or_else(|| "gif".to_string());
    
    let mut temp_file = NamedTempFile::with_suffix(&format!(".{}", file_extension))
        .map_err(|e| MonochoraError::Io(e))?;
    
    let bytes = response.bytes().await
        .map_err(|e| MonochoraError::Http(e))?;
    
     if bytes.is_empty() {
        return Err(MonochoraError::Io(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Downloaded file is empty"
            )
        ));
    }
    
    temp_file.write_all(&bytes)
        .map_err(|e| MonochoraError::Io(e))?;
    
    let temp_path = temp_file.into_temp_path();
    let final_path = temp_path.keep()
        .map_err(|e| MonochoraError::Io(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to persist temporary file: {}", e)
            )
        ))?;
    
    info!("Downloaded successfully to temporary file: {}", final_path.display());
    
    Ok(final_path)
}

fn get_file_extension_from_url(url: &Url) -> Option<String> {
    let path_segments = url.path_segments()?;
    let last_segment = path_segments.last()?;
    
    let dot_pos = last_segment.rfind('.')?;
    let extension = &last_segment[dot_pos + 1..];
    
    match extension.to_lowercase().as_str() {
        "gif" | "png" | "jpg" | "jpeg" | "webp" => {
            Some(extension.to_lowercase())
        }
        _ => None
    }
}

pub fn is_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

pub async fn get_input_path(input: &str) -> Result<PathBuf> {
    if is_url(input) {
        download_gif_from_url(input).await
    } else {
        let path = PathBuf::from(input);
        
         if !path.exists() {
            return Err(MonochoraError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Local file does not exist: {}", path.display())
                )
            ));
        }
        
         if !path.is_file() {
            return Err(MonochoraError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Path is not a file: {}", path.display())
                )
            ));
        }
        
        debug!("Using local file: {}", path.display());
        Ok(path)
    }
}
