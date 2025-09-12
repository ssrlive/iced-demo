use iced::{
    Length,
    widget::{button, column, container, row, text},
    window,
};

mod common_assets;

pub(crate) type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub(crate) const APP_NAME: &str = "MyApp";

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

fn main() -> Result<(), BoxedError> {
    // Create the tray menu
    let menu = tray_icon::menu::Menu::new();
    let show_item = tray_icon::menu::MenuItem::new("Show", true, None);
    let quit_item = tray_icon::menu::MenuItem::new("Quit", true, None);
    menu.append(&show_item)?;
    menu.append(&quit_item)?;

    // Create the tray icon
    let img = image::load_from_memory(common_assets::MAIN_ICON)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = tray_icon::Icon::from_rgba(rgba.into_raw(), width, height)?;
    let attrs = tray_icon::TrayIconAttributes {
        icon: Some(icon),
        menu: Some(Box::new(menu)),
        tooltip: Some(APP_NAME.to_string()),
        ..Default::default()
    };
    let _tray_icon = tray_icon::TrayIcon::new(attrs)?;
    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();
    std::thread::spawn(move || {
        for event in tray_icon::menu::MenuEvent::receiver() {
            if event.id() == &show_id {
                println!("Show clicked");
            } else if event.id() == &quit_id {
                println!("Quit clicked");
                std::process::exit(0);
            }
        }
    });

    iced::application(APP_NAME, update, view)
        .window(window::Settings {
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .subscription(|_state| window::events().map(|(_id, event)| Message::WindowEvent(event)))
        .run()?;
    Ok(())
}
