use super::ApplicationError;
use crossterm::event::KeyCode;
use std::sync::mpsc;

#[allow(dead_code)]
fn test6() -> Result<(), ApplicationError> {
    let sin = std::io::stdin();
    let bk = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut term = tui::Terminal::new(bk)?;
    let mut liststate = tui::widgets::ListState::default();
    term.clear()?;
    let (tx, rx) = mpsc::channel();
    let (endtx, endrx) = mpsc::channel();
    let (out_end_tx, out_end_rx) = mpsc::channel();
    let t1 = {
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

