use asset_loader::AssetLoaderPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
    App::new()
        // built-in plugins
        .add_plugins(
            DefaultPlugins, //.set(WindowPlugin {
                            // primary_window: Some(Window {
                            // mode: WindowMode::Fullscreen,
                            // ..default()
                            // }),
                            // ..default()
                            // }))
        )
        .add_plugins(WorldInspectorPlugin::default())
        // custom plugins
        .add_plugins((
            CameraPlugin,
            Playerplugin,
            MapPlugin,
            AssetLoaderPlugin,
            PhysicsPlugin,
        ))
        .run();
}
