use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));

#[derive(Component, Reflect, Resource, Default)]
#[reflect(Resource)]
struct Tile {
    id: u32,
}

#[derive(Component, Reflect, Resource, Default)]
#[reflect(Resource)]
struct MapObject {
    id: u32,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WorldInspectorPlugin::default(),
        ))
        .init_resource::<Tile>()
        .register_type::<Tile>()
        .init_resource::<MapObject>()
        .register_type::<MapObject>()
        .add_systems(Startup, setup)
        .run();
}

fn draw_level(
    commands: &mut Commands,
    server: &Res<AssetServer>,
    level: &Level,
    tilesets: &Vec<Tileset>,
    entities: &Vec<EntityDef>,
) {
    let mut layer_z_index = 0.0;

    // Draw tile layers
    for layer_instance in &level.layer_instances {
        if layer_instance.layer_type == "Tiles" {
            if let Some(tileset_uid) = layer_instance.tileset_def_uid {
                if let Some(tileset) = tilesets.iter().find(|ts| ts.uid == tileset_uid) {
                    let mut rel_path = tileset.rel_path.clone();
                    // Trim leading "../" if it exists
                    if rel_path.starts_with("../") {
                        rel_path = rel_path[3..].to_string();
                    }

                    let tile_size =
                        Vec2::new(tileset.tile_grid_size as f32, tileset.tile_grid_size as f32);
                    let tileset_handle: Handle<Image> = server.load(&rel_path);

                    for grid_tile in &layer_instance.grid_tiles {
                        let tile_position = Vec3::new(
                            grid_tile.px[0] as f32,
                            -(grid_tile.px[1] as f32),
                            layer_z_index,
                        );

                        let rect_min = Vec2::new(grid_tile.src[0] as f32, grid_tile.src[1] as f32);
                        let rect_max = rect_min + tile_size;

                        commands
                            .spawn(SpriteBundle {
                                texture: tileset_handle.clone(),
                                sprite: Sprite {
                                    custom_size: Some(tile_size),
                                    rect: Some(Rect {
                                        min: rect_min,
                                        max: rect_max,
                                    }),
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: tile_position,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Tile { id: grid_tile.t });
                    }
                }
            }
            // Increment z-index for the next layer
            layer_z_index += 1.0;
        }
    }

    // Draw entities with a higher z-index to ensure they are on top
    for layer_instance in &level.layer_instances {
        if layer_instance.layer_type == "Entities" {
            for entity_instance in &layer_instance.entity_instances {
                let entity_position = Vec3::new(
                    entity_instance.px[0] as f32,
                    -(entity_instance.px[1] as f32),
                    layer_z_index + 1.0, // Ensure entities are drawn on top of all tile layers
                );
                if let Some(entity_def) = entities
                    .iter()
                    .find(|e| e.identifier == entity_instance.identifier)
                {
                    commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::srgba(1.0, 0.0, 0.0, 1.0),
                                custom_size: Some(Vec2::new(20.0, 20.0)),
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: entity_position,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(MapObject { id: entity_def.uid });
                }
            }
        }
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    println!("{:?}", *PROJECT);

    for level in &PROJECT.levels {
        draw_level(
            &mut commands,
            &server,
            level,
            &PROJECT.tilesets,
            &PROJECT.entities,
        );
    }

    commands.spawn(Camera2dBundle::default());
}
