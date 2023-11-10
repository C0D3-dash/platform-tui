// mod app;
mod backend;
// mod components;
// mod managers;
// mod mock_components;
mod ui;

use crossterm::event::{Event as TuiEvent, EventStream};
use futures::{future::OptionFuture, select, FutureExt, StreamExt};
use rs_sdk::SdkBuilder;
use tuirealm::event::KeyEvent;

use self::{
    backend::{Backend, BackendEvent, Task},
    ui::{Ui, UiFeedback},
};

pub(crate) enum Event {
    Key(KeyEvent),
    Backend(BackendEvent),
}

#[tokio::main]
async fn main() {
    // Setup Platform SDK
    let sdk = SdkBuilder::new_mock()
        .build()
        .expect("cannot setup Platform SDK");

    let mut ui = Ui::new();
    let mut backend = Backend::new(sdk).await;

    let mut active = true;

    let mut terminal_event_stream = EventStream::new().fuse();
    let mut backend_task: OptionFuture<_> = None.into();

    while active {
        let event = select! {
            terminal_event = terminal_event_stream.next() => match terminal_event {
                None => panic!("terminal event stream closed unexpectedly"),
                Some(Err(_)) => panic!("terminal event stream closed unexpectedly"),
                Some(Ok(TuiEvent::Resize(_, _))) => {ui.redraw(); continue },
                Some(Ok(TuiEvent::Key(key_event))) => Some(Event::Key(key_event.into())),
                _ => None
            },
            backend_task_finished = backend_task => match backend_task_finished {
                Some(backend_event) => Some(
                    Event::Backend(backend_event)
                ),
                None => None
            },
        };

        let ui_feedback = event
            .map(|e| ui.on_event(backend.state(), e))
            .unwrap_or(UiFeedback::None);

        match ui_feedback {
            UiFeedback::Quit => active = false,
            UiFeedback::ExecuteTask(task) => {
                backend_task = Some(backend.run_task(task.clone()).boxed().fuse()).into();
                ui.redraw();
            }
            UiFeedback::Redraw => ui.redraw(), // TODO Debounce redraw?
            UiFeedback::None => (),
        }
    }
}
