use crossterm::ExecutableCommand;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
};
use std::io::Write;
use std::{env, fs, io, path::Path};

const VERSION: &str = "0.1.0";

// TODO
// File Size Calculating should be in another thread to NOT block the main window screen

// Struktur eines Dateieintrags
struct FileEntry {
    name: String,
    size: u64,
    is_dir: bool,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    let (cols, rows) = size()?;
    // println!("Terminal size: {} cols × {} rows", cols, rows);

    let cwd = env::current_dir()?;
    let files = list_dir(&cwd)?;

    // CLEAR TERMINAL
    stdout.execute(Clear(ClearType::All))?;

    println!("64MB File Explorer v{}", VERSION);

    let max_digits = files.iter().map(|f| num_digits(f.size)).max().unwrap_or(1);

    let name_chars = cols.saturating_sub(5 + 3 + max_digits as u16 + 10);

    let top_reserved = 2; // z. B. Titelzeile oben
    let bottom_reserved = 3; // z. B. Status + Eingabe unten

    let max_visible = rows.saturating_sub(top_reserved + bottom_reserved) as usize;

    // Nur so viele Dateien anzeigen, wie auf den Bildschirm passen
    for f in files.iter().take(max_visible) {
        print_file_entry(f, name_chars, max_digits);
    }

    let input_y = rows.saturating_sub(1); // letzte Zeile für Eingabe
    let output_y = input_y.saturating_sub(1); // 3 Zeilen darüber für Command-Ausgabe
    let mut input_buffer = String::new();
    let mut stack: Vec<String> = Vec::new();
    // Change later to 50 or other for a long history
    const MAX_HISTORY: usize = 2;

    loop {
        // Eingabezeile zeichnen
        stdout.execute(cursor::MoveTo(0, input_y))?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        write!(stdout, ":{}", input_buffer)?;
        stdout.flush()?;

        // Auf Tasteneingabe warten
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char('q') if input_buffer.is_empty() => break,
                KeyCode::Enter => {
                    if input_buffer == "quit" {
                        break;
                    } else if input_buffer == "cd" {
                        let parts: Vec<&str> = input_buffer.split_whitespace().collect();
                        if parts[1] == ".." {
                            break;
                        }
                    } else {
                        // Command in Stack speichern
                        stack.push(input_buffer.clone());
                        if stack.len() > MAX_HISTORY {
                            stack.remove(0);
                        }

                        // Command ausgeben
                        stdout.execute(cursor::MoveTo(0, output_y))?;
                        stdout.execute(Clear(ClearType::CurrentLine))?;
                        write!(
                            stdout,
                            "Command: {}",
                            stack.last().unwrap_or(&"<none>".to_string())
                        )?;
                        stdout.flush()?;

                        // Eingabezeile zurücksetzen
                        input_buffer.clear();
                        stdout.execute(cursor::MoveTo(0, input_y))?;
                        stdout.execute(Clear(ClearType::CurrentLine))?;
                        write!(stdout, ":")?;
                        stdout.flush()?;
                    }
                }

                KeyCode::Char(c) => input_buffer.push(c),
                KeyCode::Backspace => {
                    input_buffer.pop();
                }
                _ => {}
            }
        }
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
    let size_str = format!(
        "{:>width$.2} {}",
        size_val,
        size_unit,
        width = max_digits + 3
    );

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
        Err(io::Error::new(io::ErrorKind::Other, "No File"))
    }
}
