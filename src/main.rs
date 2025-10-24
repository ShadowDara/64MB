use std::{
    env,
    fs,
    io,
    path::Path,
};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, size};

const VERSION: &str = "0.1.0";

// Struktur eines Dateieintrags
struct FileEntry {
    name: String,
    size: u64,
    is_dir: bool,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;

    let (cols, rows) = size()?;
    println!("Terminal size: {} cols × {} rows", cols, rows);

    let cwd = env::current_dir()?;
    let files = list_dir(&cwd)?;

    println!("Terminal File Explorer V{}", VERSION);

    let max_digits = files
        .iter()
        .map(|f| num_digits(f.size))
        .max()
        .unwrap_or(1);

    let name_chars = cols.saturating_sub(5 + 3 + max_digits as u16 + 10);

    for f in &files {
        print_file_entry(f, name_chars, max_digits);
    }

    disable_raw_mode()?;
    Ok(())
}

/// Listet alle Dateien im Verzeichnis auf
fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
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
fn num_digits(mut n: u64) -> usize {
    if n == 0 { return 1; }
    let mut count = 0;
    while n > 0 {
        n /= 10;
        count += 1;
    }
    count
}

/// Kürzt einen String auf die angegebene Länge
fn crop_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len.saturating_sub(1)])
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
fn print_file_entry(f: &FileEntry, name_chars: u16, max_digits: usize) {
    let prefix = if f.is_dir { "[DIR]  " } else { "[FILE] " };
    let cropped = crop_string(&f.name, name_chars as usize);
    let padding = " ".repeat(name_chars.saturating_sub(f.name.len() as u16) as usize);

    let (size_val, size_unit) = human_readable_size(f.size);
    let size_str = format!("{:>width$.2} {}", size_val, size_unit, width = max_digits + 3);

    println!("{}  {}{}  {}", prefix, cropped, padding, size_str);
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
        Err(io::Error::new(io::ErrorKind::Other, "Kein File"))
    }
}


/*
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Version
pub const VERSION: &str = "0.1.0";

// Main Function
fn main() -> std::io::Result<()> {
    // START
    enable_raw_mode()?;

    // Terminal Size Cols and Rows
    let (cols, rows) = size()?;
    println!("Terminal size: {} cols × {} rows", cols, rows);

    // GET THE WORKING DIRECTORY
    let cwd = env::current_dir().unwrap();

    let files = list_dir(Path::new(&cwd))?;

    // PRINT TOPLINE
    println!("Terminal File Explorer V{}", VERSION);

    // GET BIGGEST SIZE
    let mut max_digits = 0;
    for f in &files {
        let digits = num_digits(f.size);
        if digits > max_digits {
            max_digits = digits;
        }
    }

    // TODO

    // CALCULATE WINDOW SIZE COMPLETLY
    // PREFIX = 5
    // CROPNAME
    // NAMEWHITESPACES
    // SIZESPACES
    // SIZE
    // SIZEUNIT = 3

    // CHARS
    let namecharacters = cols - 5 - 3 - max_digits as u16 - 10;

    for f in &files {
        // TODO

        // SORT OUTPUT

        // PREFIX
        let mut prefix = "";

        if f.is_dir {
            prefix = "[FILE] ";
        } else {
            prefix = "[DIR]  ";
        }

        // CROP NAME
        let cropname = crop_string(&f.name, namecharacters.into());

        // NAME FILLER WHITESPACE
        let mut nfw = String::new(); // Mutable String
        let spaces = namecharacters.saturating_sub(f.name.len() as u16); // Vermeidet negative Zahl

        for _ in 0..spaces {
            nfw.push(' '); // Ein Zeichen hinzufügen
        }

        // FILESIZE
        let sizeprefix = max_digits.saturating_sub(num_digits(f.size));

        let mut sizespaces = String::new();
        for _ in 0..sizeprefix {
            sizespaces.push(' ');
        }

        let mut calculatedsize = f.size;
        let mut sizedefinition = "  B";
        // KYLOBYTES
        if calculatedsize >= 10000 {
            calculatedsize = calculatedsize / 1024;
            sizedefinition = " KB";
            sizespaces.push(' ');
            sizespaces.push(' ');
            sizespaces.push(' ');
        }
        // MEGABYTES
        if calculatedsize >= 10000 {
            calculatedsize = calculatedsize / 1024;
            sizedefinition = " MB";
            sizespaces.push(' ');
            sizespaces.push(' ');
            sizespaces.push(' ');
        }
        // GYGABYTES
        if calculatedsize >= 10000 {
            calculatedsize = calculatedsize / 1024;
            sizedefinition = " GB";
            sizespaces.push(' ');
            sizespaces.push(' ');
            sizespaces.push(' ');
        }
        // TERABYTES
        if calculatedsize >= 10000 {
            calculatedsize = calculatedsize / 1024;
            sizedefinition = " TB";
            sizespaces.push(' ');
            sizespaces.push(' ');
            sizespaces.push(' ');
        }
        // PETABYTES
        if calculatedsize >= 10000 {
            calculatedsize = calculatedsize / 1024;
            sizedefinition = " PB";
            sizespaces.push(' ');
            sizespaces.push(' ');
            sizespaces.push(' ');
        }

        println!(
            "{}  {}{}    {}{}{}",
            // TYPE
            prefix,
            // Croppet Name
            cropname,
            // Whitespaces after the name
            nfw,
            // Spaces Before the Size
            sizespaces,
            // Size Number
            calculatedsize,
            // Size Definition
            sizedefinition
        );
    }

    // ADD LOOP
    // ADD INPUT LINE ADD THE END

    // END
    disable_raw_mode()?;
    Ok(())
}

#[derive(Debug)]
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
}

// Function to get the whole content of a Directory
fn list_dir(path: &Path) -> std::io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let name = entry.file_name().to_string_lossy().into_owned();

        entries.push(FileEntry {
            name,
            path: entry.path(),
            is_dir,
            size: folder_size(&entry.path()).unwrap(),
        });
    }

    Ok(entries)
}

// Function to crop a String
fn crop_string(s: &str, max_len: usize) -> String {
    let cropped = s.chars().take(max_len).collect::<String>();
    cropped
}

// Stellen einer Zahl ausrechnen
fn num_digits(mut n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut digits = 0;
    while n > 0 {
        digits += 1;
        n /= 10;
    }
    digits
}
*/