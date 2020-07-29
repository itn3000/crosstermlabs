use super::ApplicationError;
use crossterm::event::KeyCode;

#[allow(dead_code)]
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
