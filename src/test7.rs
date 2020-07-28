use tokio::runtime;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

use super::ApplicationError;
use crossterm::event::KeyCode;
use tui::widgets::ListState;

async fn test7_stdin_worker(mut tx: mpsc::Sender<String>, mut out_end_rx: mpsc::Receiver<bool>, mut endtx: mpsc::Sender<bool>) -> Result<(), ApplicationError> {
    let mut buf = String::new();
    let mut sin = tokio::io::stdin();
    loop {
        if out_end_rx.try_recv().is_ok() {
            break;
        }
        let bytesread = sin.read_to_string(&mut buf).await?;
        if bytesread == 0 {
            break;
        }
        for s in buf.split("\n") {
            match tx.send(s.to_string()).await {
                Ok(_) => {},
                Err(_) => break
            };
        }
        buf.clear();
    }
    endtx.try_send(true).unwrap_or_default();
    Ok(())
}

async fn test7_stdout_worker<TermBackend>(mut term: tui::Terminal<TermBackend>, mut rx: mpsc::Receiver<String>, mut endrx: mpsc::Receiver<bool>, mut liststate: tui::widgets::ListState, mut out_end_tx: mpsc::Sender<bool>) -> Result<(), ApplicationError> 
    where TermBackend: tui::backend::Backend {
    let mut data: Vec<String> = Vec::new();
    let mut current_offset = 0usize;
    loop {
        let termsize = term.size()?;
        if endrx.try_recv().is_err() {
            loop {
                if let Some(s) = rx.recv().await {
                    data.push(s);
                } else {
                    break;
                }
                if (current_offset + termsize.height as usize) <= data.len() {
                    break;
                }
            }
        }
        let chunks = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([
                tui::layout::Constraint::Percentage(90),
                tui::layout::Constraint::Length(1),
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
            let message = tui::widgets::Block::default().title(format!("{}, {}", current_offset, data.len()));
            f.render_widget(message, chunks[1]);
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
                    if current_offset + list_size < data.len() {
                        current_offset += list_size
                    } else if data.len() > list_size {
                        current_offset = data.len() - list_size
                    }
                }
                KeyCode::Char(v) => print!("{}", v),
                _ => println!("unknown event"),
            },
            _ => println!("another event"),
        }
    }
    out_end_tx.try_send(true).unwrap_or_default();
    term.clear()?;
    Ok(())
}

async fn test7_internal() -> Result<(), ApplicationError> {
    let bk = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut term = tui::Terminal::new(bk)?;
    let liststate = tui::widgets::ListState::default();
    term.clear()?;
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(64);
    let (endtx, endrx) = tokio::sync::mpsc::channel::<bool>(1);
    let (out_end_tx, out_end_rx) = tokio::sync::mpsc::channel::<bool>(1);
    let t1 = test7_stdin_worker(tx, out_end_rx, endtx);
    let t2 = {
        test7_stdout_worker(term, rx, endrx, liststate, out_end_tx)
    };
    let (r1, r2) = futures::join!(t1, t2);
    r1.unwrap();
    r2.unwrap();
    Ok(())
}

#[allow(dead_code)]
pub fn test7() -> Result<(), ApplicationError> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(test7_internal())?;
    // match t1.join() {
    //     Ok(_) => println!("thread1 OK"),
    //     Err(e) => println!("thread1 ng: {:?}", e)
    // };
    // match t2.join() {
    //     Ok(_) => println!("thread2 OK"),
    //     Err(e) => println!("thread2 ng: {:?}", e)
    // };
    Ok(())
}
