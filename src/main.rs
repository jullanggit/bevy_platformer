// Conditionally compile the import for development builds only.
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use asset_loader::AssetLoaderPlugin;
use bevy::prelude::*;
use camera::CameraPlugin;
use map::MapPlugin;
use physics::PhysicsPlugin;
use player::Playerplugin;

mod asset_loader;
mod camera;
mod map;
mod physics;
mod player;

fn main() {
    let mut app = App::new();
    // built-in plugins
    app.add_plugins(
        DefaultPlugins, //.set(WindowPlugin {
                        // primary_window: Some(Window {
                        // mode: WindowMode::Fullscreen,
                        // ..default()
                        // }),
                        // ..default()
                        // }))
    );
    // Conditionally add the WorldInspectorPlugin in development builds
    #[cfg(debug_assertions)]
    app.add_plugins(WorldInspectorPlugin::default());
    // custom plugins
    app.add_plugins((
        CameraPlugin,
        Playerplugin,
        MapPlugin,
        AssetLoaderPlugin,
        PhysicsPlugin,
    ));

    app.run();
}
