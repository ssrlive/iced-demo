use iced::{
    Length, Theme,
    widget::{button, column, container, row, text},
    window,
};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, mpsc::Receiver},
};

mod common_assets;
mod data_table;

pub(crate) type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub(crate) const APP_NAME: &str = "MyApp";

#[derive(Debug, Default, Clone)]
struct AppState {
    show_confirm: bool,
    main_table: data_table::Table,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEvent(window::Event),

    TrayIconEvent(tray_icon::menu::MenuId),
    ConfirmExit,
    CancelExit,
    TbMsg(data_table::TableMessage),
    Noop,
}

fn update(state: &mut AppState, message: Message) {
    match message {
        Message::WindowEvent(window::Event::CloseRequested) => {
            state.show_confirm = true;
        }
        Message::WindowEvent(event) => {
            // log the window event for debugging and forward it to the table handler
            log::info!("WindowEvent: {event:?}");
            state.main_table.on_window_event_debug(&format!("{event:?}"));
        }
        Message::ConfirmExit => {
            std::process::exit(0);
        }
        Message::TrayIconEvent(ref menu_id) => {
            handle_tray_icon_event(menu_id);
        }
        Message::TbMsg(msg) => state.main_table.update(msg),
        Message::CancelExit => {
            state.show_confirm = false;
        }
        Message::Noop => {}
    }
}

fn view(state: &'_ AppState) -> iced::Element<'_, Message> {
    let content: iced::Element<'_, Message> = if state.show_confirm {
        container(column![
            text("Are you sure you want to exit?"),
            row![
                button(text("Confirm")).on_press(Message::ConfirmExit),
                button(text("Cancel")).on_press(Message::CancelExit)
            ]
        ])
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        // Use the data table view here and map its messages into our app Message::TbMsg
        state.main_table.view().map(Message::TbMsg)
    };
    content
}

const STR_SHOW: &str = "Show";
const STR_QUIT: &str = "Quit";

static TRAY_ICON_MENU_ITEM_IDS: LazyLock<Arc<Mutex<HashMap<&str, tray_icon::menu::MenuId>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

fn handle_tray_icon_event(event_id: &tray_icon::menu::MenuId) {
    log::info!("Event ID: {event_id:?}");
    let quit_id = TRAY_ICON_MENU_ITEM_IDS.lock().unwrap().get(&STR_QUIT).cloned();
    let show_id = TRAY_ICON_MENU_ITEM_IDS.lock().unwrap().get(&STR_SHOW).cloned();
    if let Some(show_id) = show_id
        && event_id == &show_id
    {
        log::info!("Show clicked");
        // Here you would typically send a message to your application to show or hide the window
    }
    if let Some(quit_id) = quit_id
        && event_id == &quit_id
    {
        log::info!("Quit clicked");
        std::process::exit(0);
    }
}

fn main() -> Result<(), BoxedError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let (tx, rx) = std::sync::mpsc::channel();

    // Global static receiver
    static TRAY_ICON_EVENT_RECEIVER: LazyLock<Mutex<Option<Receiver<tray_icon::menu::MenuId>>>> = LazyLock::new(|| Mutex::new(None));
    *TRAY_ICON_EVENT_RECEIVER.lock().unwrap() = Some(rx);

    // Create the tray menu
    let menu = tray_icon::menu::Menu::new();
    let show_item = tray_icon::menu::MenuItem::new(STR_SHOW, true, None);
    let quit_item = tray_icon::menu::MenuItem::new(STR_QUIT, true, None);
    menu.append(&show_item)?;
    menu.append(&quit_item)?;

    TRAY_ICON_MENU_ITEM_IDS.lock().unwrap().insert(STR_SHOW, show_item.id().clone());
    TRAY_ICON_MENU_ITEM_IDS.lock().unwrap().insert(STR_QUIT, quit_item.id().clone());

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
    std::thread::spawn(move || {
        for event in tray_icon::menu::MenuEvent::receiver() {
            if let Err(e) = tx.send(event.id().clone()) {
                log::error!("Failed to send tray icon event: {e}");
            }
        }
    });

    iced::application(AppState::default, update, view)
        .window(window::Settings {
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .subscription(move |_state| {
            iced::Subscription::batch(vec![
                window::events().map(|(_id, event)| Message::WindowEvent(event)),
                iced::time::every(std::time::Duration::from_millis(100)).map(move |_| {
                    match TRAY_ICON_EVENT_RECEIVER.lock().unwrap().as_ref().unwrap().try_recv() {
                        Ok(event_id) => Message::TrayIconEvent(event_id),
                        Err(_) => Message::Noop,
                    }
                }),
            ])
        })
        .theme(Theme::Dark)
        .run()?;
    Ok(())
}
