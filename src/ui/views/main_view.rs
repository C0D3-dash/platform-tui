//! The view a user sees on application start.

use tuirealm::event::{Key, KeyEvent, KeyModifiers};

use super::identities_view::IdentitiesScreenController;
use crate::{
    backend::Task,
    ui::{
        form::{ComposedInput, Field, FormController, FormStatus, Input, SelectInput, TextInput},
        screen::{ScreenCommandKey, ScreenController, ScreenFeedback, ScreenToggleKey},
    },
};

const COMMAND_KEYS: [ScreenCommandKey; 6] = [
    ScreenCommandKey::new("q", "Quit"),
    ScreenCommandKey::new("i", "Identities"),
    ScreenCommandKey::new("c", "Contracts"),
    ScreenCommandKey::new("w", "Wallet"),
    ScreenCommandKey::new("v", "Version Upgrade"),
    ScreenCommandKey::new("t", "Test form"),
];

pub(crate) struct MainScreenController;

impl ScreenController for MainScreenController {
    fn name(&self) -> &'static str {
        "Main menu"
    }

    fn init_text(&self) -> &'static str {
        r#"Welcome to Platform TUI!

Use keys listed in the section below to switch screens and execute commands.
Some of them require signature and are disabled until an identity key is loaded.

Italics are used to mark flags.
Bold italics are flags that are enabled.

Text inputs with completions support both arrows and Ctrl+n / Ctrl+p keys for selection.
Use Ctrl+q to go back from completion list or once again to leave input at all.
"#
    }

    fn command_keys(&self) -> &[ScreenCommandKey] {
        COMMAND_KEYS.as_ref()
    }

    fn toggle_keys(&self) -> &[ScreenToggleKey] {
        [].as_ref()
    }

    fn on_event(&mut self, key_event: KeyEvent) -> ScreenFeedback {
        match key_event {
            KeyEvent {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            } => ScreenFeedback::Quit,
            KeyEvent {
                code: Key::Char('i'),
                modifiers: KeyModifiers::NONE,
            } => ScreenFeedback::NextScreen(Box::new(IdentitiesScreenController::new())),
            KeyEvent {
                code: Key::Char('c'),
                modifiers: KeyModifiers::NONE,
            } => ScreenFeedback::None,
            KeyEvent {
                code: Key::Char('w'),
                modifiers: KeyModifiers::NONE,
            } => ScreenFeedback::None,
            KeyEvent {
                code: Key::Char('v'),
                modifiers: KeyModifiers::NONE,
            } => ScreenFeedback::None,
            KeyEvent {
                code: Key::Char('t'),
                modifiers: KeyModifiers::NONE,
            } => ScreenFeedback::Form(Box::new(TestFormController::new())),
            _ => ScreenFeedback::None,
        }
    }
}

#[derive(Clone, strum::Display, Debug)]
enum TestVariants {
    Yeet,
    Swag,
    Kek,
    Lol,
}

struct TestFormController {
    input: ComposedInput<(
        Field<TextInput>,
        Field<SelectInput<TestVariants>>,
        Field<TextInput>,
    )>,
}

impl TestFormController {
    fn new() -> Self {
        Self {
            input: ComposedInput::new((
                Field::new("lol", TextInput::new("lol placeholder")),
                Field::new(
                    "kek",
                    SelectInput::new(vec![TestVariants::Yeet, TestVariants::Lol]),
                ),
                Field::new(
                    "cheburek",
                    TextInput::new_init_value("cheburek placeholder", "lao gan ma"),
                ),
            )),
        }
    }
}

impl FormController for TestFormController {
    fn on_event(&mut self, event: KeyEvent) -> crate::ui::form::FormStatus {
        match self.input.on_event(event) {
            crate::ui::form::InputStatus::Done(result) => {
                FormStatus::Done(Task::RenderData(format!("{:?}", result)))
            }
            crate::ui::form::InputStatus::Redraw => FormStatus::Redraw,
            crate::ui::form::InputStatus::None => FormStatus::None,
        }
    }

    fn step_view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::tui::prelude::Rect) {
        self.input.view(frame, area)
    }

    fn form_name(&self) -> &'static str {
        "Test form"
    }

    fn step_name(&self) -> &'static str {
        self.input.step_name()
    }

    fn step_index(&self) -> u8 {
        self.input.step_index()
    }

    fn steps_number(&self) -> u8 {
        self.input.steps_number()
    }
}
