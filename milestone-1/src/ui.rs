//! The systems and components for the user interface.

use bevy::{color::palettes::tailwind, prelude::*};

/// Baseline text size, in pixels
pub const TEXT_SIZE: f32 = 20.0;
/// Background color for the game
pub const BG_COLOR: Color = Color::BLACK;
/// Padding ardound the board, in pixels
pub const PADDING: f32 = 30.0;

/// Marker component for the title text
#[derive(Component)]
pub struct TitleText;

/// Create a text bundle that contains the given bundle.
pub fn mk_text(text: impl Into<String>, rest: impl Bundle, font_size: Option<f32>) -> impl Bundle {
    (
        Text(text.into()),
        TextFont {
            font_size: font_size.unwrap_or(TEXT_SIZE),
            ..Default::default()
        },
        rest,
    )
}

/// Setup the game UI.
pub fn setup_ui(mut commands: Commands) {
    commands.spawn(mk_text(
        "BLOCK DROPPER 3000!",
        (
            TextColor(Color::from(tailwind::PINK_300)),
            Node {
                margin: auto().horizontal(),
                top: px(PADDING / 3.0),
                ..default()
            },
            TitleText,
            UiTransform::from_translation(Val2::px(0.0, 10.0)),
        ),
        Some(TEXT_SIZE + 10.0),
    ));

    commands.spawn((mk_text(
        r"Help:
left, right: Move
down, space: Drop
z:           Enable/disable
             hard drop
x:           Swap hold
",
        Node {
            top: percent(10),
            left: percent(5),
            ..default()
        },
        Some(10.0),
    ),));
}

/// Animate the title text
pub fn animate_title(mut title: Single<&mut UiTransform, With<TitleText>>, time: Res<Time>) {
    title.rotation = Rot2::degrees((time.elapsed_secs() * 2.0).sin() * 10.0);
}
