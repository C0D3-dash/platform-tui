use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    // // The top level container:
    // let outer_block_size = frame.size();
    // let outer_block = Block::default()
    //     .title("Template")
    //     .title_alignment(Alignment::Center)
    //     .borders(Borders::ALL);

    // App layout
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(frame.size());

    //    frame.render_widget(outer_block, outer_block_size);
    frame.render_widget(
        Block::default()
            .title("Platform Explorer")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.counter
        ))
        .block(
            Block::default()
                .title("Commands")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL),
        ),
        layout[1],
    );
}
