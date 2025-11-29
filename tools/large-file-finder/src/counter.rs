//! Line counter for Large File Finder
//!
//! This module handles counting lines in files.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Count the number of lines in a file
pub fn count_lines(path: &Path) -> Result<usize> {
    let file = File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let reader = BufReader::new(file);
    
    let line_count = reader.lines().count();
    
    Ok(line_count)
}

/// Count lines efficiently by reading bytes and counting newlines
/// This is faster for very large files but may be less accurate for files with unusual line endings
#[allow(dead_code)]
pub fn count_lines_fast(path: &Path) -> Result<usize> {
    use std::io::Read;
    
    let file = File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);
    
    let mut count = 0;
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        count += buffer[..bytes_read].iter().filter(|&&b| b == b'\n').count();
    }
    
    Ok(count)
}

/// Check if a file is likely a text file (not binary)
pub fn is_text_file(path: &Path) -> bool {
    use std::io::Read;
    
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 512];
        
        if let Ok(bytes_read) = reader.read(&mut buffer) {
            // Check for null bytes which typically indicate binary content
            return !buffer[..bytes_read].contains(&0);
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_count_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();
        
        let count = count_lines(file.path()).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_lines_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let count = count_lines(file.path()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_lines_fast() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();
        
        let count = count_lines_fast(file.path()).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_is_text_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "This is a text file").unwrap();
        
        assert!(is_text_file(file.path()));
    }

    #[test]
    fn test_is_not_text_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        
        assert!(!is_text_file(file.path()));
    }
}
