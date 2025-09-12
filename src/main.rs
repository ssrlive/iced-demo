use iced::{
    Length,
    widget::{button, column, container, row, text},
    window,
};

#[derive(Debug, Default, Clone)]
struct AppState {
    show_confirm: bool,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEvent(window::Event),
    RequestExit,
    ConfirmExit,
    CancelExit,
}

fn update(state: &mut AppState, message: Message) {
    match message {
        Message::WindowEvent(window::Event::CloseRequested) => {
            state.show_confirm = true;
        }
        Message::RequestExit => {
            state.show_confirm = true;
        }
        Message::ConfirmExit => {
            std::process::exit(0);
        }
        Message::CancelExit => {
            state.show_confirm = false;
        }
        _ => {}
    }
}

fn view(state: &'_ AppState) -> iced::Element<'_, Message> {
    let content = if state.show_confirm {
        container(column![
            text("Are you sure you want to exit?"),
            row![
                button(text("Confirm")).on_press(Message::ConfirmExit),
                button(text("Cancel")).on_press(Message::CancelExit)
            ]
        ])
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    } else {
        container(column![
            text("Hello from Iced!"),
            button(text("Quit")).on_press(Message::RequestExit)
        ])
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    };
    content.into()
}

fn main() -> iced::Result {
    iced::application("MyApp", update, view)
        .window(window::Settings {
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .subscription(|_state| window::events().map(|(_id, event)| Message::WindowEvent(event)))
        .run()
}
