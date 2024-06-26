use crate::app::AppResult;
use crate::metrics::{self, Metrics};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::sync::mpsc;
use std::thread;

/// Terminal events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// Metrix
    Metrics(Metrics),
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let tx_key = tx.clone();
        thread::spawn(move || loop {
            match event::read().expect("unable to read event") {
                CrosstermEvent::Key(e) => {
                    if e.kind == KeyEventKind::Press {
                        tx_key.send(Event::Key(e))
                    } else {
                        Ok(())
                    }
                }
                CrosstermEvent::Mouse(e) => tx_key.send(Event::Mouse(e)),
                CrosstermEvent::Resize(w, h) => tx_key.send(Event::Resize(w, h)),
                CrosstermEvent::FocusGained => Ok(()),
                CrosstermEvent::FocusLost => Ok(()),
                CrosstermEvent::Paste(_) => unimplemented!(),
            }
            .expect("failed to send terminal event")
        });
        let tx_metrics = tx.clone();

        thread::spawn(move || loop {
            let mut metrics = Metrics::default();
            metrics.collect_metrics();
            if let Err(msg) = tx_metrics.send(Event::Metrics(metrics)) {
                panic!("{}", msg);
            }
        });
        Self {
            sender: tx,
            receiver: rx,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }
}
