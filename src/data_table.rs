use iced::Length;
use iced::font;
use iced::time::{Duration, hours, minutes};
use iced::widget::{button, center_x, center_y, column, container, row, scrollable, slider, text, tooltip};
use iced::{Center, Element, Fill, Font};

#[derive(Debug, Clone)]
pub enum TableMessage {
    PaddingChanged(f32, f32),
    SeparatorChanged(f32, f32),
    ShowDetails(usize),
    HideDetails,
    HideContext,
}

#[derive(Debug, Clone)]
pub struct Table {
    events: Vec<Event>,
    padding: (f32, f32),
    separator: (f32, f32),
    selected: Option<usize>,
    last_cursor: Option<(f32, f32)>,
    context_menu: Option<(usize, f32, f32)>,
}

impl Table {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: TableMessage) {
        match message {
            TableMessage::PaddingChanged(x, y) => self.padding = (x, y),
            TableMessage::SeparatorChanged(x, y) => self.separator = (x, y),
            TableMessage::ShowDetails(idx) => self.selected = Some(idx),
            TableMessage::HideDetails => self.selected = None,
            TableMessage::HideContext => self.context_menu = None,
        }
    }

    // Handle window events forwarded from main as a debug string. This is a heuristic
    // approach: we parse debug output to extract cursor positions and right-click presses.
    pub fn on_window_event_debug(&mut self, debug: &str) {
        // update cursor position from debug strings that include moved/cursor coordinates
        // different Iced/backends may produce slightly different Debug representations
        // so we look for either "CursorMoved" or "Moved" and parse x/y floats if present.
        if debug.contains("CursorMoved") || debug.contains("Moved(") || debug.contains("MovedPoint") {
            // try to parse any "x: <float>" and "y: <float>" occurrences
            let mut x_opt: Option<f32> = None;
            let mut y_opt: Option<f32> = None;

            if let Some(x_idx) = debug.find("x:") {
                let tail = &debug[x_idx + 2..].trim_start();
                let num_str: String = tail.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
                if let Ok(xv) = num_str.trim().parse::<f32>() {
                    x_opt = Some(xv);
                }
            }

            if let Some(y_idx) = debug.find("y:") {
                let tail = &debug[y_idx + 2..].trim_start();
                let num_str: String = tail.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
                if let Ok(yv) = num_str.trim().parse::<f32>() {
                    y_opt = Some(yv);
                }
            }

            if let (Some(xv), Some(yv)) = (x_opt, y_opt) {
                self.last_cursor = Some((xv, yv));
                log::debug!("Cursor updated to: {:?}", self.last_cursor);
            }
        }

        // detect right-click press in debug string
        // detect right-click press in debug string. Try several keywords that vary by backend
        if (debug.contains("MouseInput") || debug.contains("MouseButton") || debug.contains("ButtonPressed") || debug.contains("Pressed"))
            && debug.contains("Right")
        {
            // use last_cursor to compute row index
            if let Some((x, y)) = self.last_cursor {
                let header_h = 36.0;
                let row_h = 36.0;
                if y > header_h {
                    let idx = ((y - header_h) / row_h).floor() as usize;
                    if idx < self.events.len() {
                        log::debug!("Context menu set for idx {} at ({},{})", idx, x, y);
                        self.context_menu = Some((idx, x, y));
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, TableMessage> {
        // Build a manual table: header + rows so we can add per-row buttons (for context/details)
        let bold = |header| {
            text(header).font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
        };

        let mut rows = column![
            row![
                bold("Name").width(300),
                bold("Time").width(80),
                bold("Price").width(80),
                bold("Rating").width(80),
                // actions
                bold("")
            ]
            .spacing(10)
            .padding(5)
        ];

        for (i, event) in self.events.iter().enumerate() {
            let minutes = event.duration.as_secs() / 60;
            let time_text = text(format!("{minutes} min")).style(if minutes > 90 { text::warning } else { text::default });
            let price_text = if event.price > 0.0 {
                text(format!("${:.2}", event.price)).style(if event.price > 100.0 { text::warning } else { text::default })
            } else {
                text("Free").style(text::success).width(Fill).center()
            };
            let rating_text = text(format!("{:.2}", event.rating)).style(if event.rating > 4.7 {
                text::success
            } else if event.rating < 2.0 {
                text::danger
            } else {
                text::default
            });

            let details_button = button(text("⋮")).on_press(TableMessage::ShowDetails(i));

            rows = rows.push(
                row![
                    text(&event.name).width(300),
                    time_text.width(80),
                    price_text.width(80),
                    rating_text.width(80),
                    details_button
                ]
                .spacing(10)
                .padding(5),
            );
        }

        let controls = {
            let labeled_slider = |label, range: std::ops::RangeInclusive<f32>, (x, y), on_change: fn(f32, f32) -> TableMessage| {
                row![
                    text(label).font(Font::MONOSPACE).size(14).width(100),
                    tooltip(
                        slider(range.clone(), x, move |x| on_change(x, y)),
                        text!("{x:.0}px").font(Font::MONOSPACE).size(10),
                        tooltip::Position::Left
                    ),
                    tooltip(
                        slider(range, y, move |y| on_change(x, y)),
                        text!("{y:.0}px").font(Font::MONOSPACE).size(10),
                        tooltip::Position::Right
                    ),
                ]
                .spacing(10)
                .align_y(Center)
            };

            column![
                labeled_slider("Padding", 0.0..=30.0, self.padding, TableMessage::PaddingChanged),
                labeled_slider("Separator", 0.0..=5.0, self.separator, TableMessage::SeparatorChanged)
            ]
            .spacing(10)
            .width(400)
        };

        // Compose main column: table rows + controls
        let mut main_col = column![
            center_y(scrollable(center_x(rows)).spacing(10)).padding(10),
            center_x(controls).padding(10).style(container::dark)
        ]
        .spacing(10);

        // Render context menu if requested
        // Render context menu if requested. We try to position it near last_cursor when possible.
        if let Some((idx, x, _y)) = self.context_menu {
            let menu = container(column![
                button(text("Show details")).on_press(TableMessage::ShowDetails(idx)),
                button(text("Close menu")).on_press(TableMessage::HideContext)
            ])
            .padding(8)
            .style(container::dark)
            .width(200);

            // approximate horizontal position: if x is known and large use left padding to shift menu
            let positioned = if x > 0.0 {
                // convert x into a left padding amount (clamped)
                let pad = (x - 100.0).clamp(0.0, 600.0) as u16;
                container(menu).padding(pad)
            } else {
                container(menu)
            };

            main_col = main_col.push(positioned.padding(10));
        }

        // Render modal dialog when a row is selected
        // If modal is active, render it as a full-screen overlay so it behaves like a blocking modal.
        if let Some(idx) = self.selected
            && let Some(ev) = self.events.get(idx)
        {
            let modal_body = container(column![
                text(format!("Name: {}", ev.name)).size(18),
                text(format!("Duration: {} min", ev.duration.as_secs() / 60)),
                text(format!("Price: ${:.2}", ev.price)),
                text(format!("Rating: {:.2}", ev.rating)),
                row![button(text("Close")).on_press(TableMessage::HideDetails)]
            ])
            .padding(16)
            .width(400)
            .style(container::dark);

            // full-screen semi-transparent backdrop + centered modal body
            let backdrop = container(column![center_y(center_x(modal_body))])
                .width(Length::Fill)
                .height(Length::Fill)
                .style(container::dark);

            // Return only the overlay to ensure it visually blocks the rest of the UI.
            return backdrop.into();
        }

        main_col.into()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            events: Event::list(),
            padding: (10.0, 5.0),
            separator: (1.0, 1.0),
            selected: None,
            last_cursor: None,
            context_menu: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Event {
    name: String,
    duration: Duration,
    price: f32,
    rating: f32,
}

impl Event {
    fn list() -> Vec<Self> {
        vec![
            Event {
                name: "Get lost in a hacker bookstore".to_owned(),
                duration: hours(2),
                price: 0.0,
                rating: 4.9,
            },
            Event {
                name: "Buy vintage synth at Noisebridge flea market".to_owned(),
                duration: hours(1),
                price: 150.0,
                rating: 4.8,
            },
            Event {
                name: "Eat a questionable hot dog at 2AM".to_owned(),
                duration: minutes(20),
                price: 5.0,
                rating: 1.7,
            },
            Event {
                name: "Ride the MUNI for the story".to_owned(),
                duration: minutes(60),
                price: 3.0,
                rating: 4.1,
            },
            Event {
                name: "Scream into the void from Twin Peaks".to_owned(),
                duration: minutes(40),
                price: 0.0,
                rating: 4.9,
            },
            Event {
                name: "Buy overpriced coffee and feel things".to_owned(),
                duration: minutes(25),
                price: 6.5,
                rating: 4.5,
            },
            Event {
                name: "Attend an underground robot poetry slam".to_owned(),
                duration: hours(1),
                price: 12.0,
                rating: 4.8,
            },
            Event {
                name: "Browse cursed tech at a retro computer fair".to_owned(),
                duration: hours(2),
                price: 10.0,
                rating: 4.7,
            },
            Event {
                name: "Try to order at a secret ramen place with no sign".to_owned(),
                duration: minutes(50),
                price: 14.0,
                rating: 4.6,
            },
            Event {
                name: "Join a spontaneous rooftop drone rave".to_owned(),
                duration: hours(3),
                price: 0.0,
                rating: 4.9,
            },
            Event {
                name: "Sketch a stranger at Dolores Park".to_owned(),
                duration: minutes(45),
                price: 0.0,
                rating: 4.4,
            },
            Event {
                name: "Visit the Museum of Obsolete APIs".to_owned(),
                duration: hours(1),
                price: 9.99,
                rating: 4.2,
            },
            Event {
                name: "Chase the last working payphone".to_owned(),
                duration: minutes(35),
                price: 0.25,
                rating: 4.0,
            },
            Event {
                name: "Trade zines with a punk on BART".to_owned(),
                duration: minutes(30),
                price: 3.5,
                rating: 4.7,
            },
            Event {
                name: "Get a tattoo of the Git logo".to_owned(),
                duration: hours(1),
                price: 200.0,
                rating: 4.6,
            },
        ]
    }
}
