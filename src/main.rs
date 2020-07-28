extern crate crossterm;
extern crate cursive;
extern crate scopeguard;
extern crate thiserror;
extern crate tokio;
extern crate tui;
extern crate futures;

use crossterm::event::Event;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::ExecutableCommand;
use crossterm::{
    execute,
    terminal::{size, ScrollDown, ScrollUp, SetSize},
};
use std::error::Error;
use std::io::stdout;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::io::{Seek, SeekFrom};
use std::sync::mpsc;
use std::thread;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncReadExt;

mod test7;

#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    #[error("crossterm error:{0:?}")]
    Crossterm(#[from] crossterm::ErrorKind),
    #[error("io error:{0:?}")]
    Io(#[from] std::io::Error),
    #[error("channel send error:{0:?}")]
    ChannelSendError(#[from] mpsc::SendError<String>),
    #[error("tokio channel string send error:{0:?}")]
    TokioChannelStringError(#[from] tokio::sync::mpsc::error::SendError<String>),
    #[error("tokio channel bool send error:{0:?}")]
    TokioChannelBoolError(#[from] tokio::sync::mpsc::error::SendError<bool>),
}

fn test1() -> Result<(), ApplicationError> {
    let (tx, rx) = mpsc::channel();
    let (col, height) = crossterm::terminal::size().unwrap();
    println!("{}, {}", col, height);
    let in_thread = thread::spawn(move || -> std::io::Result<()> {
        let sin = std::io::stdin();
        let mut bufin = String::new();
        bufin.reserve(4096);
        loop {
            let bytesread = sin.read_line(&mut bufin)?;
            if bytesread > 0 {
                match tx.send(bufin.clone()) {
                    Ok(v) => v,
                    Err(e) => return Err(IoError::new(ErrorKind::Other, format!("{:?}", e))),
                }
                bufin.clear();
            } else {
                break;
            }
        }
        Ok(())
    });
    let out_thread = thread::spawn(move || -> std::io::Result<()> {
        let mut sout = std::io::stdout();
        for recv in rx {
            sout.write(&recv.as_bytes())?;
        }
        Ok(())
    });
    in_thread.join().unwrap().unwrap();
    out_thread.join().unwrap().unwrap();
    Ok(())
}

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

fn test3() -> Result<(), ApplicationError> {
    let bk = cursive::backends::crossterm::Backend::init()?;
    let mut cur = cursive::Cursive::new(|| cursive::backends::crossterm::Backend::init().unwrap());
    cur.add_global_callback('q', |s| s.quit());
    cur.add_layer(cursive::views::TextView::new("press <q> to quit"));
    cur.run();
    Ok(())
}

fn test4() -> Result<(), ApplicationError> {
    let bk = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut term = tui::Terminal::new(bk)?;
    let mut liststate = tui::widgets::ListState::default();
    term.clear()?;
    let mut current_selected = 0usize;
    loop {
        term.draw(|f| {
            let chunks = tui::layout::Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    tui::layout::Constraint::Percentage(50),
                    tui::layout::Constraint::Percentage(50),
                ])
                .split(f.size());
            let items: Vec<tui::widgets::ListItem> = (0..10)
                .map(|i| {
                    let spans = tui::text::Spans::from(format!("line {}", i));
                    tui::widgets::ListItem::new(spans)
                })
                .collect();
            let itemlist = tui::widgets::List::new(items).highlight_symbol(">>");
            liststate.select(Some(current_selected));
            f.render_stateful_widget(itemlist, chunks[0], &mut liststate);
        })?;
        term.hide_cursor()?;
        let key_event = crossterm::event::read()?;
        match key_event {
            crossterm::event::Event::Key(v) => match v.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') => {
                    if current_selected < 9 {
                        current_selected += 1
                    }
                }
                KeyCode::Char('k') => {
                    if current_selected > 0 {
                        current_selected -= 1
                    }
                }
                KeyCode::Char(v) => print!("{}", v),
                _ => println!("unknown event"),
            },
            _ => println!("another event"),
        }
    }

    Ok(())
}
fn test5() -> Result<(), ApplicationError> {
    let bk = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut term = tui::Terminal::new(bk)?;
    let mut liststate = tui::widgets::ListState::default();
    term.clear()?;
    let mut current_offset = 0usize;
    let mut data: Vec<String> = Vec::new();
    for i in 0..100 {
        data.push(format!("line{}", i));
    }
    loop {
        let termsize = term.size()?;
        term.hide_cursor()?;
        let chunks = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([
                tui::layout::Constraint::Percentage(90),
                tui::layout::Constraint::Percentage(10),
            ])
            .split(termsize);
        let list_size = if termsize.height < chunks[0].height {
            termsize.height as usize
        } else {
            chunks[0].height as usize
        };
        term.draw(|f| {
            let endpoint = if current_offset + list_size < data.len() {
                current_offset + list_size
            } else {
                data.len()
            };
            let items: Vec<tui::widgets::ListItem> = (current_offset..endpoint)
                .map(|i| {
                    let spans = tui::text::Spans::from(data[i].as_str());
                    tui::widgets::ListItem::new(spans)
                })
                .collect();
            let itemlist = tui::widgets::List::new(items).highlight_symbol(">>");
            liststate.select(Some(10));
            f.render_widget(itemlist, chunks[0]);
        })?;
        let key_event = crossterm::event::read()?;
        match key_event {
            crossterm::event::Event::Key(v) => match v.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') => {
                    if current_offset < data.len() - list_size {
                        current_offset += 1
                    }
                }
                KeyCode::Char('k') => {
                    if current_offset > 0 {
                        current_offset -= 1
                    }
                }
                KeyCode::Char(' ') => {
                    if current_offset < data.len() - list_size * 2 {
                        current_offset += list_size
                    } else {
                        current_offset = data.len() - list_size
                    }
                }
                KeyCode::Char(v) => print!("{}", v),
                _ => println!("unknown event"),
            },
            _ => println!("another event"),
        }
    }

    Ok(())
}

fn test6() -> Result<(), ApplicationError> {
    let sin = std::io::stdin();
    let bk = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut term = tui::Terminal::new(bk)?;
    let mut liststate = tui::widgets::ListState::default();
    term.clear()?;
    let mut current_offset = 0usize;
    let mut data: Vec<String> = Vec::new();
    let (tx, rx) = mpsc::channel();
    let (endtx, endrx) = mpsc::channel();
    let (out_end_tx, out_end_rx) = mpsc::channel();
    let mtx = std::sync::Arc::new(std::sync::Mutex::new(0));
    let t1 = {
        let mtx = mtx.clone();
        std::thread::spawn(move || -> Result<(), ApplicationError> {
            let mut buf = String::new();
            scopeguard::defer!(
                endtx.send(true).unwrap();
            );
            loop {
                if out_end_rx.try_recv().is_ok() {
                    break;
                }
                let bytesread = sin.read_line(&mut buf)?;
                if bytesread == 0 {
                    break;
                }
                tx.send(buf.clone())?;
                buf.clear();
            }
            Ok(())
        })
    };
    let t2 = {
        std::thread::spawn(move || -> Result<(), ApplicationError> {
            let mut data: Vec<String> = Vec::new();
            let mut current_offset = 0usize;
            scopeguard::defer!(out_end_tx.send(true).unwrap_or_default());
            loop {
                let termsize = term.size()?;
                if endrx.try_recv().is_err() {
                    loop {
                        if let Ok(v) = rx.try_recv() {
                            data.push(v);
                        } else if endrx.try_recv().is_ok() {
                            break;
                        }
                        if (current_offset + termsize.height as usize) >= data.len() {
                            break;
                        }
                    }
                }
                let chunks = tui::layout::Layout::default()
                    .direction(tui::layout::Direction::Vertical)
                    .constraints([
                        tui::layout::Constraint::Percentage(90),
                        tui::layout::Constraint::Percentage(10),
                    ])
                    .split(termsize);
                let list_size = if termsize.height < chunks[0].height {
                    termsize.height as usize
                } else {
                    chunks[0].height as usize
                };
                term.draw(|f| {
                    let endpoint = if current_offset + list_size < data.len() {
                        current_offset + list_size
                    } else {
                        data.len()
                    };
                    let items: Vec<tui::widgets::ListItem> = (current_offset..endpoint)
                        .map(|i| {
                            let spans = tui::text::Spans::from(data[i].as_str());
                            tui::widgets::ListItem::new(spans)
                        })
                        .collect();
                    let itemlist = tui::widgets::List::new(items).highlight_symbol(">>");
                    liststate.select(Some(10));
                    f.render_widget(itemlist, chunks[0]);
                })?;
                let key_event = crossterm::event::read()?;
                match key_event {
                    crossterm::event::Event::Key(v) => match v.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => {
                            if data.len() > list_size && current_offset < data.len() - list_size {
                                current_offset += 1
                            }
                        }
                        KeyCode::Char('k') => {
                            if current_offset > 0 {
                                current_offset -= 1
                            }
                        }
                        KeyCode::Char(' ') => {
                            if current_offset < data.len() - list_size * 2 {
                                current_offset += list_size
                            } else {
                                current_offset = data.len() - list_size
                            }
                        }
                        KeyCode::Char(v) => print!("{}", v),
                        _ => println!("unknown event"),
                    },
                    _ => println!("another event"),
                }
            }
            term.clear()?;
            Ok(())
        })
    };
    match t1.join() {
        Ok(_) => println!("thread1 OK"),
        Err(e) => println!("thread1 ng: {:?}", e)
    };
    match t2.join() {
        Ok(_) => println!("thread2 OK"),
        Err(e) => println!("thread2 ng: {:?}", e)
    };
    Ok(())
}


fn main() -> Result<(), ApplicationError> {
    // test2()?;
    // test3()?;
    // test4()?;
    // test5()?;
    // test6()?;
    test7::test7()?;
    Ok(())
}
