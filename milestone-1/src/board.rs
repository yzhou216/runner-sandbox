//! The tetris board setup

use std::time::Duration;

use crate::there_is_collision;
use crate::ui::{BG_COLOR, PADDING};

use super::data::*;
use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};

/// The main board containing visible tiles.
#[derive(Resource)]
pub struct Board {
    // Visible tiles as entities
    cells: Vec<Vec<Entity>>,
}

/// Side-length of an *unscaled* tile in pixels.
pub const TILE_SIDE_LEN: f32 = 40.0;

/// Amount of time before a tile is locked.
pub const LOCKDOWN_DURATION: Duration = GameState::initial_drop_interval();

/// An event signalling that the game is over.
#[derive(Event)]
pub struct GameOver;

/// An block.  This is one of:
/// - an obstacle (leftovers of an inactive tetromino)
/// - a block used in the preview and held views.
#[derive(Component, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    /// Coordinates of this block
    pub cell: Cell,
    /// Color of this block
    pub color: Color,
}

/// A timer to count down when a piece must be inactivated after it can't be pushed down
#[derive(Resource)]
pub struct LockdownTimer(Option<Timer>);

impl LockdownTimer {
    // Advance the timer. Start it if it hasn't been started.
    fn start_or_advance(&mut self, time: Res<Time<Fixed>>) {
        match &mut self.0 {
            Some(timer) => {
                timer.tick(time.delta());
            }
            None => self.0 = Some(Timer::new(LOCKDOWN_DURATION, TimerMode::Once)),
        }
    }

    // Has this timer just gone off?
    fn just_finished(&self) -> bool {
        self.0.as_ref().is_some_and(Timer::just_finished)
    }

    fn reset(&mut self) {
        self.0 = None;
    }
}

// Create a logical tile to insert into a board.
//
// Board width and board height are the information of the board this tile is
// placed in.
fn mk_tile(
    x: u32,
    y: u32,
    board_width: u32,
    board_height: u32,
    tile_mesh: Handle<Mesh>,
    tile_material: Handle<ColorMaterial>,
) -> impl Bundle {
    (
        Mesh2d(tile_mesh),
        MeshMaterial2d(tile_material),
        (
            Text2d(format!("{x},{y}")),
            TextFont {
                font_size: 12.0,
                ..Default::default()
            },
        ),
        Transform::from_xyz(
            (x as f32 - board_width as f32 / 2.0) * TILE_SIDE_LEN + PADDING / 2.0,
            (y as f32 - board_height as f32 / 2.0) * TILE_SIDE_LEN + PADDING / 2.0,
            0.0,
        )
        .with_scale(Vec3::splat(0.8)),
    )
}

/// The calculated window height based on the board size.
pub const fn window_height() -> f32 {
    TILE_SIDE_LEN * BOARD_HEIGHT as f32 + PADDING * 2.0 + crate::ui::TEXT_SIZE * 2.0
}

/// The calculated window width based on the board size.
pub const fn window_width() -> f32 {
    window_height()
}

/// Set up a side window to show the next piece or the hold area.
pub fn spawn_side_window(
    transform: Transform,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    commands: &mut Commands,
    title: &str,
    marker: impl Component + Copy,
) {
    commands
        .spawn((transform, Visibility::default()))
        .with_children(|parent| {
            (0..5).for_each(|y| {
                (0..5).for_each(|x| {
                    parent.spawn((
                        mk_tile(x, y, 5, 5, mesh.clone(), material.clone()),
                        Block {
                            cell: Cell(x as i32, y as i32),
                            color: BG_COLOR,
                        },
                        marker,
                    ));
                })
            });

            parent.spawn((
                Transform::from_xyz(-4. * TILE_SIDE_LEN * 0.5, 5. * TILE_SIDE_LEN * 0.5, -1.0),
                Text2d::new(title),
            ));
        });
}

/// Set up the window. Only used when not testing.
#[cfg(all(not(feature = "ci"), not(feature = "test")))]
pub fn setup_window(mut window: Single<&mut Window>) {
    window.resolution.set(window_height(), window_height());
}

/// Create the board and initialize game data
pub fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Background color
    commands.insert_resource(ClearColor(Color::BLACK));

    // Set up the camera
    commands.spawn(Camera2d);

    let mesh = meshes.add(Rectangle::new(TILE_SIDE_LEN, TILE_SIDE_LEN));
    let material = materials.add(BG_COLOR);

    // Set up the board
    let cells = (0..BOARD_HEIGHT)
        .map(|y| {
            (0..BOARD_WIDTH)
                .map(|x| {
                    commands
                        .spawn(mk_tile(
                            x,
                            y,
                            BOARD_WIDTH,
                            BOARD_HEIGHT,
                            mesh.clone(),
                            material.clone(),
                        ))
                        .id()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    commands.insert_resource(Board { cells });

    spawn_side_window(
        Transform::from_xyz(
            (BOARD_WIDTH + 5) as f32 * TILE_SIDE_LEN * 0.5 + PADDING,
            window_height() * 0.5 - 5.0 * TILE_SIDE_LEN * 0.5 - PADDING,
            0.0,
        ),
        mesh.clone(),
        material.clone(),
        &mut commands,
        "Next",
        Next,
    );

    commands.add_observer(exit_on_game_over);
    commands.insert_resource(LockdownTimer(None));
}

/// Apply the update to the tetromino unless that leaves it out of bounds or
/// overlapping an obstacle.
///
/// Returns whether the update was applied.
fn update_if_legal(
    tetromino: &mut Tetromino,
    obstacles: &Query<&Block, With<Obstacle>>,
    update: impl FnOnce(&mut Tetromino),
) -> bool {
    let mut updated = *tetromino;
    update(&mut updated);
    let legal = !there_is_collision(&updated, obstacles.as_readonly());
    if legal {
        *tetromino = updated;
    }
    legal
}

/// Handle user input for the purposes of moving and/or rotating the tetromino.
///
/// The actions happen in this order when several keys are pressed together:
/// down, left, right, rotate.  An action that would leave the tetromino in an
/// illegal position is skipped.
pub fn handle_user_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    tetromino: Option<Single<&mut Tetromino, With<Active>>>,
    obstacles: Query<&Block, With<Obstacle>>,
    state: Res<GameState>,
) {
    let Some(mut tetromino) = tetromino else {
        return;
    };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        // Drop one row at a time so that the piece stops at the first collision
        for _ in 0..state.manual_drop_gravity {
            if !update_if_legal(&mut tetromino, &obstacles, |t| t.shift(0, -1)) {
                break;
            }
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        update_if_legal(&mut tetromino, &obstacles, |t| t.shift(-1, 0));
    }

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        update_if_legal(&mut tetromino, &obstacles, |t| t.shift(1, 0));
    }

    // Up and space together still rotate only once
    if keyboard.any_just_pressed([KeyCode::ArrowUp, KeyCode::Space]) {
        update_if_legal(&mut tetromino, &obstacles, Tetromino::rotate);
    }
}

/// Drop the piece whenever the gravity timer goes off
pub fn gravity(
    tetromino: Option<Single<&mut Tetromino, With<Active>>>,
    obstacles: Query<&Block, With<Obstacle>>,
    mut state: ResMut<GameState>,
    time: Res<Time<Fixed>>,
) {
    // Keep the timer running even when there is no piece to drop
    let fired = state.gravity_timer.tick(time.delta()).just_finished();
    if fired && let Some(mut tetromino) = tetromino {
        update_if_legal(&mut tetromino, &obstacles, |t| t.shift(0, -1));
    }
}

/// Check if the active tetromino cannot move down. If so, deactivate it.
///
/// A tetromino that stays unable to move down for the lockdown duration is
/// turned into obstacles, one for each of its cells.
pub fn deactivate_if_stuck(
    mut commands: Commands,
    active: Option<Single<(Entity, &Tetromino), With<Active>>>,
    obstacles: Query<&Block, With<Obstacle>>,
    mut lockdown: ResMut<LockdownTimer>,
    time: Res<Time<Fixed>>,
) {
    let Some(active) = active else {
        return;
    };
    let (entity, tetromino) = active.into_inner();

    let mut below = *tetromino;
    below.shift(0, -1);
    if !there_is_collision(&below, obstacles.as_readonly()) {
        // Restart the countdown while the piece can still move down
        lockdown.reset();
        return;
    }

    lockdown.start_or_advance(time);
    if lockdown.just_finished() {
        lockdown.reset();
        commands.entity(entity).despawn();
        tetromino.cells().iter().for_each(|&cell| {
            commands.spawn((
                Block {
                    cell,
                    color: tetromino.color,
                },
                Obstacle,
            ));
        });
    }
}

/// Offset that moves a tetromino from the origin to the middle of a side window.
pub const SIDE_WINDOW_OFFSET: i32 = 2;

/// Show the piece at the front of the bag in the next window, replacing the
/// piece that is shown there now.
pub fn show_next_piece(commands: &mut Commands, state: &mut GameState, shown: Option<Entity>) {
    if let Some(shown) = shown {
        commands.entity(shown).despawn();
    }
    let mut preview = state.bag.peek();
    preview.shift(SIDE_WINDOW_OFFSET, SIDE_WINDOW_OFFSET);
    commands.spawn((preview, Next));
}

/// Spawn the next tetromino if there is no active tetromino.  This should also
/// update the next tetromino window.
///
/// The game is over when there is no room left for the new tetromino.
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    active: Option<Single<&Active>>,
    next: Option<Single<Entity, (With<Next>, With<Tetromino>)>>,
    obstacles: Query<&Block, With<Obstacle>>,
) {
    if active.is_some() {
        return;
    }

    // Move the piece from the origin to the top center of the visible board
    let mut tetromino = state.bag.next_tetromino();
    tetromino.shift(
        BOARD_WIDTH as i32 / 2 - 1,
        BOARD_HEIGHT as i32 - 1 - tetromino.bounds().top,
    );
    if there_is_collision(&tetromino, obstacles.as_readonly()) {
        commands.trigger(GameOver);
        return;
    }
    commands.spawn((tetromino, Active));

    // Replace the preview with the piece that comes after this one
    show_next_piece(&mut commands, &mut state, next.map(|next| *next));
}

/// Redraw the board.
pub fn redraw_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tetrominoes: Query<&Tetromino, With<Active>>,
    obstacles: Query<&Block, With<Obstacle>>,
    mut board: ResMut<Board>,
) {
    // Color of the tile for each visible cell that is lit up, either by the
    // active tetromino or by an obstacle
    let colors: HashMap<Entity, Color> = tetrominoes
        .iter()
        .flat_map(|tetromino| {
            tetromino
                .cells()
                .iter()
                .map(|&cell| (cell, tetromino.color))
        })
        .chain(obstacles.iter().map(|block| (block.cell, block.color)))
        .filter(|(cell, _)| cell.is_visible())
        .map(|(Cell(x, y), color)| (board.cells[y as usize][x as usize], color))
        .collect();

    // re-draw the whole board
    for entity in board.cells.iter_mut().flat_map(|row| row.iter_mut()) {
        commands.entity(*entity).insert(MeshMaterial2d(
            materials.add(colors.get(entity).copied().unwrap_or(BG_COLOR)),
        ));
    }
}

/// Redraw the side board with the given marker component.
pub fn redraw_side_board<Marker: Component>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut side_board: Query<(&mut Block, Entity), With<Marker>>,
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    if let Some(tetromino) = tetromino {
        side_board.iter_mut().for_each(|(mut block, entity)| {
            // Light up the cells covered by the tetromino and clear the rest
            block.color = if tetromino.cells().contains(&block.cell) {
                tetromino.color
            } else {
                BG_COLOR
            };
            commands
                .entity(entity)
                .insert(MeshMaterial2d(materials.add(block.color)));
        });
    }
}

/// Trigger game over when Escape is pressed
pub fn game_over_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.just_released(KeyCode::Escape) {
        commands.trigger(GameOver);
    }
}

/// Exit the program when the game over event is triggered
pub fn exit_on_game_over(_: On<GameOver>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
