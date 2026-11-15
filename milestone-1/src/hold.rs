//! Functionality for holding a piece

use crate::Game;
use crate::there_is_collision;
use crate::ui::{BG_COLOR, PADDING};

use super::board::*;
use super::data::*;
use bevy::prelude::*;

/// Whether this tetromino is the one being held
#[derive(Component, Copy, Clone)]
pub struct Hold;

/// Whether the player asked to swap the held piece and the swap is still pending.
///
/// The key press is noticed in `Update`, whereas the swap itself runs in
/// `FixedUpdate`, so the request is remembered until the next fixed step.
#[derive(Resource, Default)]
pub struct SwapRequest(bool);

/// Number of times a piece that is swapped in may move up to get out of a collision.
const MAX_SWAP_KICKS: i32 = 4;

/// Cell of the board that contains the center of the tetromino.
fn center_cell(tetromino: &Tetromino) -> Cell {
    let (x, y) = tetromino.center();
    Cell(x.floor() as i32, y.floor() as i32)
}

/// Remember that the player pressed the key to swap the held piece.
fn request_swap(keyboard: Res<ButtonInput<KeyCode>>, mut request: ResMut<SwapRequest>) {
    if keyboard.just_pressed(KeyCode::KeyX) {
        request.0 = true;
    }
}

/// Swap the current piece and the piece in the hold window on user input.
///
/// If no piece is held, then take the next piece as the active piece and move
/// the current piece to the hold window.
///
/// This system also has to make sure that the swap is legal and kick the piece
/// up by up to 4 times until the swap is legal.  If that is not possible, then
/// abort the swap.
pub fn swap_hold(
    mut commands: Commands,
    mut request: ResMut<SwapRequest>,
    mut state: ResMut<GameState>,
    active: Option<Single<(Entity, &Tetromino), With<Active>>>,
    held: Option<Single<(Entity, &Tetromino), With<Hold>>>,
    next: Option<Single<Entity, (With<Next>, With<Tetromino>)>>,
    obstacles: Query<&Block, With<Obstacle>>,
) {
    // A request is used up even if the swap turns out to be impossible
    if !std::mem::take(&mut request.0) {
        return;
    }
    let Some(active) = active else {
        return;
    };
    let (active_entity, outgoing) = active.into_inner();
    let held = held.map(Single::into_inner);

    // The incoming piece is the held one, or else the one waiting in the next
    // window.  Do not take it out of the bag until the swap is known to work.
    let incoming = held.map_or_else(
        || {
            let mut preview = state.bag.peek();
            preview.shift(SIDE_WINDOW_OFFSET, SIDE_WINDOW_OFFSET);
            preview
        },
        |(_, tetromino)| *tetromino,
    );

    // Place the incoming piece so that its center cell is the outgoing one's
    let Cell(out_x, out_y) = center_cell(outgoing);
    let Cell(in_x, in_y) = center_cell(&incoming);
    let mut placed = incoming;
    placed.shift(out_x - in_x, out_y - in_y);

    // Kick the piece up until it fits, or give up
    let Some(swapped_in) = (0..=MAX_SWAP_KICKS)
        .map(|kick| {
            let mut kicked = placed;
            kicked.shift(0, kick);
            kicked
        })
        .find(|kicked| !there_is_collision(kicked, obstacles.as_readonly()))
    else {
        return;
    };

    // Move the outgoing piece to the hold window, keeping its orientation
    let mut stored = *outgoing;
    stored.shift(SIDE_WINDOW_OFFSET - out_x, SIDE_WINDOW_OFFSET - out_y);

    commands.entity(active_entity).insert(swapped_in);
    match held {
        Some((entity, _)) => {
            commands.entity(entity).insert(stored);
        }
        None => {
            commands.spawn((stored, Hold));
            // Show the piece after the one that was just taken
            state.bag.next_tetromino();
            show_next_piece(&mut commands, &mut state, next.map(|next| *next));
        }
    }
}

/// Set up the window that shows the held piece.
pub fn setup_hold_window(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(TILE_SIDE_LEN, TILE_SIDE_LEN));
    let material = materials.add(BG_COLOR);

    crate::board::spawn_side_window(
        Transform::from_xyz(
            (BOARD_WIDTH + 5) as f32 * TILE_SIDE_LEN * 0.5 + PADDING,
            -window_height() * 0.5 + 5.0 * TILE_SIDE_LEN * 0.5 + PADDING,
            0.0,
        ),
        mesh.clone(),
        material.clone(),
        &mut commands,
        "Hold",
        Hold,
    );
}

/// Plugin that lets the player hold a piece and swap it back later.
pub struct HoldPlugin;

impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SwapRequest>()
            .add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(FixedUpdate, swap_hold.in_set(Game))
            .add_systems(
                Update,
                (request_swap, redraw_side_board::<Hold>).in_set(Game),
            );
    }
}
