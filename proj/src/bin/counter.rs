// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

use iced::Element;
use iced::widget::{button, column, text};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} {}: {}", record.level(), record.target(), record.args());
        }
    }
    fn flush(&self) {}
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement),
        ]
        .padding(20)
        .spacing(10)
        .into()
    }
}

fn main() -> iced::Result {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Info);
    eprintln!("counter: starting iced");
    // DejaVu Sans in place of the system's sans-serif with
    // ROSTEST_EMBEDDED_FONT set, to tell the system's fonts apart from
    // iced's text
    let embedded = std::env::var_os("ROSTEST_EMBEDDED_FONT").is_some();
    let application = iced::application(Counter::default, Counter::update, Counter::view)
        .title("rostest counter")
        .font(include_bytes!(env!("ROSTEST_FONT")).as_slice());
    if embedded {
        application
            .default_font(iced::Font::with_name("DejaVu Sans"))
            .run()
    } else {
        application.run()
    }
}
