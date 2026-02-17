use bevy::prelude::*;
use std::sync::{Mutex, OnceLock};
use wasm_bindgen::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct GameSession {
    pub game_id: Option<String>,
}

#[derive(Component)]
struct GameIdLabel;

static GAME_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn game_id_slot() -> &'static Mutex<Option<String>> {
    GAME_ID.get_or_init(|| Mutex::new(None))
}

fn current_game_id() -> Option<String> {
    game_id_slot().lock().ok().and_then(|slot| slot.clone())
}

#[wasm_bindgen]
pub fn set_game_id(game_id: String) {
    if let Ok(mut slot) = game_id_slot().lock() {
        *slot = Some(game_id);
    }
}

/// Build the Bevy [`App`] with all plugins and systems.
pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy-canvas".to_string()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: true,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(GameSession {
        game_id: current_game_id(),
    })
    .add_systems(Startup, setup)
    .add_systems(Update, sync_game_id_label);

    app
}

/// WASM entry point — called automatically when the module is loaded.
#[wasm_bindgen(start)]
pub fn wasm_main() {
    build_app().run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_session: Res<GameSession>,
) {
    let label = game_session.game_id.as_ref().map_or_else(
        || "Game UUID: not set".to_string(),
        |id| format!("Game UUID: {id}"),
    );

    if let Some(game_id) = &game_session.game_id {
        info!("Loaded game session for game {game_id}");
    }

    commands.spawn((
        Text::new(label),
        Node {
            position_type: PositionType::Absolute,
            top: px(12.0),
            left: px(12.0),
            ..default()
        },
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        GameIdLabel,
    ));

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn sync_game_id_label(
    mut game_session: ResMut<GameSession>,
    mut labels: Query<&mut Text, With<GameIdLabel>>,
) {
    let latest = current_game_id();
    if latest == game_session.game_id {
        return;
    }

    game_session.game_id = latest;

    let text = game_session.game_id.as_ref().map_or_else(
        || "Game UUID: not set".to_string(),
        |id| format!("Game UUID: {id}"),
    );

    for mut label in &mut labels {
        *label = Text::new(text.clone());
    }
}
