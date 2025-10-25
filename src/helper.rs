use std::{fs, io, path::Path};
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

// Struktur eines Dateieintrags
pub struct FileEntry {
    name: String,
    pub size: u64,
    is_dir: bool,
}

/// Listet alle Dateien im Verzeichnis auf
pub fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            size: folder_size(&entry.path()).unwrap(),
            is_dir: metadata.is_dir(),
        });
    }
    Ok(entries)
}

/// Ermittelt die Anzahl der Ziffern einer Zahl
pub fn num_digits(mut n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    while n > 0 {
        n /= 10;
        count += 1;
    }
    count
}

/// Kürzt einen String auf die angegebene Länge

fn crop_string(s: &str, max_width: usize) -> String {
    if s.width() > max_width {
        let mut result = String::new();
        let mut current = 0;
        for ch in s.chars() {
            let w = ch.width().unwrap_or(0);
            if current + w >= max_width.saturating_sub(1) {
                break;
            }
            result.push(ch);
            current += w;
        }
        result.push('…');
        result
    } else {
        s.to_string()
    }
}

/// Formatiert die Dateigröße in human-readable Format (B, KB, MB, GB, …)
fn human_readable_size(size: u64) -> (f64, &'static str) {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    (size, UNITS[unit_index])
}

/// Gibt einen Dateieintrag formatiert aus
pub fn print_file_entry(f: &FileEntry, name_width: u16, max_digits: usize) {
    let prefix = if f.is_dir { "[DIR]" } else { "[FILE]" };
    let cropped = crop_string(&f.name, name_width as usize);
    let (size_val, size_unit) = human_readable_size(f.size);
    let size_str = format!(
        "{:>width$.2} {}",
        size_val,
        size_unit,
        width = max_digits + 3
    );

    println!(
        "{:<7} {:<width$} {}",
        prefix,
        cropped,
        size_str,
        width = name_width as usize
    );
}

/// Berechnet die Größe eines Ordners rekursiv in Bytes
/// If its not a Folder, the function will call the filesize function
fn folder_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                size += folder_size(&entry_path)?; // rekursiver Aufruf
            } else {
                size += metadata.len(); // Dateigröße hinzufügen
            }
        }
    } else {
        size = file_size(path).unwrap();
    }

    Ok(size)
}

/// Gibt die Größe einer Datei in Bytes zurück
fn file_size(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        Ok(metadata.len())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "No File"))
    }
}
