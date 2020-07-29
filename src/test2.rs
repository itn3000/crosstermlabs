use std::io::stdout;
use std::io::{Write};
use crossterm::{
    execute,
    terminal::{size, ScrollDown, ScrollUp, SetSize},
    style::Print,
    ExecutableCommand,
    event::KeyCode,
    event::Event,
};
use super::ApplicationError;

#[allow(dead_code)]
fn test2() -> Result<(), ApplicationError> {
    let (cols, rows) = size()?;
    execute!(
        stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        SetSize(cols, 20),
        ScrollUp(10),
        Print(format!("hello world:{}, {}\n", cols, rows)),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
    )?;
    {
        let mut sin = std::io::stdout();
        for _ in 0..10 {
            sin.execute(Print('c'))?
                .execute(ScrollUp(1))?
                .execute(crossterm::cursor::MoveLeft(0))?;
        }
    }
    loop {
        match crossterm::event::read()? {
            Event::Key(k) => {
                match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => execute!(std::io::stdout(), ScrollDown(1)),
                    KeyCode::Char('k') => execute!(std::io::stdout(), ScrollUp(1)),
                    KeyCode::Char(v) => {
                        execute!(std::io::stdout(), Print(v), crossterm::cursor::MoveLeft(0))
                    }
                    _ => Ok(()),
                }?;
            }
            _ => execute!(std::io::stdout(), Print("another event"))?,
        };
    }
    execute!(stdout(), SetSize(cols, rows))?;
    Ok(())
}

