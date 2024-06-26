use mactop_rs::app::{App, AppResult};
use mactop_rs::event::{Event, EventHandler};
use mactop_rs::handler::handle_key_events;
use mactop_rs::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

fn main() -> AppResult<()> {
    // Create an application.
    let mut app = App::new();
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(2500);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    // Start the main loop.
    while app.running {
        // app.collect_metrics();
        // Render the user interface.
        tui.draw(&mut app)?;
        // println!("{:?}", app.cpu_metrics);
        // Handle events.
        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Metrics(metrics) => {
                app.cpu_w.push(metrics.cpu_metrics.package_w.to_owned());
                if app.cpu_w.len() > 25 {
                    app.cpu_w.remove(0);
                }
                app.metrics = metrics;
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
