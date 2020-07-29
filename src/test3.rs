use super::ApplicationError;

#[allow(dead_code)]
fn test3() -> Result<(), ApplicationError> {
    // let bk = cursive::backends::crossterm::Backend::init()?;
    let mut cur = cursive::Cursive::new(|| cursive::backends::crossterm::Backend::init().unwrap());
    cur.add_global_callback('q', |s| s.quit());
    cur.add_layer(cursive::views::TextView::new("press <q> to quit"));
    cur.run();
    Ok(())
}

