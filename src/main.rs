// Conditionally compile the import for development builds only.
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use asset_loader::AssetLoaderPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*};
use camera::CameraPlugin;
use map::MapPlugin;
use physics::PhysicsPlugin;
use player::Playerplugin;
#[cfg(target_family = "wasm")]
use wasm::WasmPlugin;

mod asset_loader;
mod camera;
mod map;
mod physics;
mod player;
#[cfg(target_family = "wasm")]
mod wasm;

fn main() {
    let mut app = App::new();
    // disable checking for .meta files
    app.insert_resource(AssetMetaCheck::Never);

    // built-in plugins
    app.add_plugins(DefaultPlugins);

    // Conditionally add the WorldInspectorPlugin in development builds
    #[cfg(debug_assertions)]
    app.add_plugins(WorldInspectorPlugin::default());

    // wasm stuff
    #[cfg(target_family = "wasm")]
    app.add_plugins(WasmPlugin);

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
