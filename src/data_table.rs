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
}

#[derive(Debug, Clone)]
pub struct Table {
    events: Vec<Event>,
    padding: (f32, f32),
    separator: (f32, f32),
    selected: Option<usize>,
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

        // If a row is selected, show its details below
        if let Some(idx) = self.selected {
            if let Some(ev) = self.events.get(idx) {
                let details = container(column![
                    text(format!("Name: {}", ev.name)),
                    text(format!("Duration: {} min", ev.duration.as_secs() / 60)),
                    text(format!("Price: ${:.2}", ev.price)),
                    text(format!("Rating: {:.2}", ev.rating)),
                    row![button(text("Close")).on_press(TableMessage::HideDetails)]
                ])
                .padding(10)
                .style(container::dark);

                main_col = main_col.push(center_x(details).padding(10));
            }
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
