use anyhow::{Result, bail};
use bytes::Bytes;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
const ALLOWED_EXTENSIONS: &[&str] = &["pdf", "jpg", "jpeg", "png", "dcm"];

pub fn validate_file(file_name: &str, file_data: &Bytes) -> Result<()> {
    // Check file size
    if file_data.len() > MAX_FILE_SIZE {
        bail!("File size exceeds maximum allowed size of 50MB");
    }
    
    if file_data.is_empty() {
        bail!("File is empty");
    }
    
    // Check file extension
    let extension = file_name
        .split('.')
        .last()
        .unwrap_or("")
        .to_lowercase();
    
    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        bail!("File type not supported. Allowed: PDF, JPG, PNG, DICOM");
    }
    
    Ok(())
}

pub fn determine_file_type(mime_type: &str) -> String {
    if mime_type.contains("pdf") {
        "pdf".to_string()
    } else if mime_type.contains("image") {
        "image".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_size() {
        let data = Bytes::from(vec![0u8; 100]);
        assert!(validate_file("test.pdf", &data).is_ok());
        
        let large_data = Bytes::from(vec![0u8; MAX_FILE_SIZE + 1]);
        assert!(validate_file("test.pdf", &large_data).is_err());
    }

    #[test]
    fn test_validate_extension() {
        let data = Bytes::from(vec![0u8; 100]);
        assert!(validate_file("test.pdf", &data).is_ok());
        assert!(validate_file("test.jpg", &data).is_ok());
        assert!(validate_file("test.exe", &data).is_err());
    }

    #[test]
    fn test_determine_file_type() {
        assert_eq!(determine_file_type("application/pdf"), "pdf");
        assert_eq!(determine_file_type("image/jpeg"), "image");
        assert_eq!(determine_file_type("image/png"), "image");
    }
}
