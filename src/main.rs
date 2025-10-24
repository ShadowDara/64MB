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

mod helper;

const VERSION: &str = "0.1.0";

// TODO
// File Size Calculating should be in another thread to NOT block the main window screen

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    let (cols, rows) = size()?;
    // println!("Terminal size: {} cols × {} rows", cols, rows);

    let cwd = env::current_dir()?;
    let files = helper::list_dir(&cwd)?;

    // CLEAR TERMINAL
    stdout.execute(Clear(ClearType::All))?;

    println!("64MB File Explorer v{}", VERSION);

    let max_digits = files.iter().map(|f| helper::num_digits(f.size)).max().unwrap_or(1);

    let name_chars = cols.saturating_sub(5 + 3 + max_digits as u16 + 10);

    let top_reserved = 2; // z. B. Titelzeile oben
    let bottom_reserved = 3; // z. B. Status + Eingabe unten

    let max_visible = rows.saturating_sub(top_reserved + bottom_reserved) as usize;

    // Nur so viele Dateien anzeigen, wie auf den Bildschirm passen
    for f in files.iter().take(max_visible) {
        helper::print_file_entry(f, name_chars, max_digits);
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
                            // DIR ÄNDERN
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

                        // Eingabezeile zurücksetzen
                        input_buffer.clear();
                        stdout.execute(cursor::MoveTo(0, input_y))?;
                        stdout.execute(Clear(ClearType::CurrentLine))?;
                        

                        write!(stdout, ":")?;
                        // stdout.flush()?;
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
