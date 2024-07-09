extern crate proc_macro;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct LDtkProject {
    defs: Defs,
    levels: Vec<Level>,
}

#[derive(Debug, Deserialize)]
struct Defs {
    tilesets: Vec<Tileset>,
    entities: Vec<EntityDef>,
    enums: Vec<EnumDef>,
}

#[derive(Debug, Deserialize)]
struct Tileset {
    uid: u32,
    relPath: Option<String>,
    pxWid: u32,
    pxHei: u32,
    tileGridSize: u32,
}

#[derive(Debug, Deserialize)]
struct EntityDef {
    identifier: String,
    uid: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct EnumDef {
    identifier: String,
    uid: u32,
    values: Vec<EnumValueDef>,
}

#[derive(Debug, Deserialize)]
struct EnumValueDef {
    id: String,
    tileRect: TileRect,
}

#[derive(Debug, Deserialize)]
struct TileRect {
    tilesetUid: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Debug, Deserialize)]
struct Level {
    identifier: String,
    pxWid: u32,
    pxHei: u32,
    layerInstances: Vec<LayerInstance>,
}

#[derive(Debug, Deserialize)]
struct LayerInstance {
    __identifier: String,
    __type: String,
    __gridSize: u32,
    __cWid: u32,
    __cHei: u32,
    __tilesetDefUid: Option<u32>,
    gridTiles: Vec<GridTile>,
    entityInstances: Vec<EntityInstance>,
}

#[derive(Debug, Deserialize)]
struct GridTile {
    px: Vec<u32>,
    src: Vec<u32>,
    f: u32,
    t: u32,
    d: Vec<u32>,
    a: u32,
}

#[derive(Debug, Deserialize)]
struct EntityInstance {
    __identifier: String,
    px: Vec<u32>,
    fieldInstances: Vec<FieldInstance>,
}

#[derive(Debug, Deserialize)]
struct FieldInstance {
    __identifier: String,
    __type: String,
    __value: serde_json::Value,
}

fn parse_ldtk_file(file_path: &str) -> LDtkProject {
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    serde_json::from_str(&contents).expect("JSON was not well-formatted")
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("generated_code.rs");

    let project = parse_ldtk_file("assets/maps/project.ldtk.json");

    let levels: Vec<TokenStream> = project.levels.iter().map(|level| {
        let identifier = &level.identifier;
        let width = level.pxWid;
        let height = level.pxHei;
        let layer_instances: Vec<TokenStream> = level.layerInstances.iter().map(|layer_instance| {
            let identifier = &layer_instance.__identifier;
            let layer_type = &layer_instance.__type;
            let grid_size = layer_instance.__gridSize;
            let cwid = layer_instance.__cWid;
            let chei = layer_instance.__cHei;
            let tileset_def_uid = layer_instance.__tilesetDefUid.map(|uid| quote! { Some(#uid) }).unwrap_or_else(|| quote! { None });
            let grid_tiles: Vec<TokenStream> = layer_instance.gridTiles.iter().map(|grid_tile| {
                let px = &grid_tile.px;
                let src = &grid_tile.src;
                let f = grid_tile.f;
                let t = grid_tile.t;
                let d = &grid_tile.d;
                let a = grid_tile.a;
                quote! {
                    GridTile { px: vec![#(#px),*], src: vec![#(#src),*], f: #f, t: #t, d: vec![#(#d),*], a: #a }
                }
            }).collect();
            let entity_instances: Vec<TokenStream> = layer_instance.entityInstances.iter().map(|entity_instance| {
                let identifier = &entity_instance.__identifier;
                let px = &entity_instance.px;
                let field_instances: Vec<TokenStream> = entity_instance.fieldInstances.iter().map(|field_instance| {
                    let identifier = &field_instance.__identifier;
                    let field_type = &field_instance.__type;
                    let value_str = field_instance.__value.to_string();
                    quote! {
                        FieldInstance { identifier: #identifier.to_string(), field_type: #field_type.to_string(), value: serde_json::from_str(#value_str).unwrap() }
                    }
                }).collect();
                quote! {
                    EntityInstance {
                        identifier: #identifier.to_string(),
                        px: vec![#(#px),*],
                        field_instances: vec![#(#field_instances),*],
                    }
                }
            }).collect();
            quote! {
                LayerInstance {
                    identifier: #identifier.to_string(),
                    layer_type: #layer_type.to_string(),
                    grid_size: #grid_size,
                    cwid: #cwid,
                    chei: #chei,
                    tileset_def_uid: #tileset_def_uid,
                    grid_tiles: vec![#(#grid_tiles),*],
                    entity_instances: vec![#(#entity_instances),*],
                }
            }
        }).collect();
        quote! {
            Level {
                identifier: #identifier.to_string(),
                width: #width,
                height: #height,
                layer_instances: vec![#(#layer_instances),*],
            }
        }
    }).collect();

    let tilesets: Vec<TokenStream> = project
        .defs
        .tilesets
        .iter()
        .map(|tileset| {
            let uid = tileset.uid;
            let rel_path = tileset.relPath.as_ref().unwrap_or(&"".to_string()).clone();
            let width = tileset.pxWid;
            let height = tileset.pxHei;
            let tile_grid_size = tileset.tileGridSize;
            quote! {
                Tileset {
                    uid: #uid,
                    rel_path: #rel_path.to_string(),
                    width: #width,
                    height: #height,
                    tile_grid_size: #tile_grid_size,
                }
            }
        })
        .collect();

    let entity_defs: Vec<TokenStream> = project
        .defs
        .entities
        .iter()
        .map(|entity_def| {
            let identifier = &entity_def.identifier;
            let uid = entity_def.uid;
            let width = entity_def.width;
            let height = entity_def.height;
            quote! {
                EntityDef {
                    identifier: #identifier.to_string(),
                    uid: #uid,
                    width: #width,
                    height: #height,
                }
            }
        })
        .collect();

    let enum_defs: Vec<TokenStream> = project
        .defs
        .enums
        .iter()
        .map(|enum_def| {
            let identifier = &enum_def.identifier;
            let uid = enum_def.uid;
            let values: Vec<TokenStream> = enum_def
                .values
                .iter()
                .map(|value| {
                    let id = &value.id;
                    let tileset_uid = value.tileRect.tilesetUid;
                    let x = value.tileRect.x;
                    let y = value.tileRect.y;
                    let w = value.tileRect.w;
                    let h = value.tileRect.h;
                    quote! {
                        EnumValueDef {
                            id: #id.to_string(),
                            tile_rect: TileRect {
                                tileset_uid: #tileset_uid,
                                x: #x,
                                y: #y,
                                w: #w,
                                h: #h,
                            },
                        }
                    }
                })
                .collect();
            quote! {
                EnumDef {
                    identifier: #identifier.to_string(),
                    uid: #uid,
                    values: vec![#(#values),*],
                }
            }
        })
        .collect();

    let generated_code = quote! {
        use once_cell::sync::Lazy;
        use serde_json;

        #[derive(Debug)]
        struct GridTile {
            px: Vec<u32>,
            src: Vec<u32>,
            f: u32,
            t: u32,
            d: Vec<u32>,
            a: u32,
        }

        #[derive(Debug)]
        struct FieldInstance {
            identifier: String,
            field_type: String,
            value: serde_json::Value,
        }

        #[derive(Debug)]
        struct EntityInstance {
            identifier: String,
            px: Vec<u32>,
            field_instances: Vec<FieldInstance>,
        }

        #[derive(Debug)]
        struct LayerInstance {
            identifier: String,
            layer_type: String,
            grid_size: u32,
            cwid: u32,
            chei: u32,
            tileset_def_uid: Option<u32>,
            grid_tiles: Vec<GridTile>,
            entity_instances: Vec<EntityInstance>,
        }

        #[derive(Debug)]
        struct Level {
            identifier: String,
            width: u32,
            height: u32,
            layer_instances: Vec<LayerInstance>,
        }

        #[derive(Debug)]
        struct Tileset {
            uid: u32,
            rel_path: String,
            width: u32,
            height: u32,
            tile_grid_size: u32,
        }

        #[derive(Debug)]
        struct EntityDef {
            identifier: String,
            uid: u32,
            width: u32,
            height: u32,
        }

        #[derive(Debug)]
        struct TileRect {
            tileset_uid: u32,
            x: u32,
            y: u32,
            w: u32,
            h: u32,
        }

        #[derive(Debug)]
        struct EnumValueDef {
            id: String,
            tile_rect: TileRect,
        }

        #[derive(Debug)]
        struct EnumDef {
            identifier: String,
            uid: u32,
            values: Vec<EnumValueDef>,
        }

        #[derive(Debug)]
        struct LDtkProject {
            levels: Vec<Level>,
            tilesets: Vec<Tileset>,
            entities: Vec<EntityDef>,
            enums: Vec<EnumDef>,
        }

        static PROJECT: Lazy<LDtkProject> = Lazy::new(|| LDtkProject {
            levels: vec![#(#levels),*],
            tilesets: vec![#(#tilesets),*],
            entities: vec![#(#entity_defs),*],
            enums: vec![#(#enum_defs),*],
        });
    };

    let mut file = File::create(dest_path).unwrap();
    write!(file, "{}", generated_code).unwrap();
}
