use super::ApplicationError;
use crossterm::event::KeyCode;

#[allow(dead_code)]
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

