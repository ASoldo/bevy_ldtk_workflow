![image](https://github.com/ASoldo/bevy_ldtk_workflow/assets/1175537/7ccb5438-43cf-4346-bad2-c30f8f388348)


# Bevy LDtk Map Parser and Code Generator

This project demonstrates how to parse a .ldtk/.json file, generate corresponding Rust code, and integrate it into a Bevy game. The generated code is written to a file during the build process and can be used within the main game code.

## Overview

The main goal of this project is to:

1. Parse a LDtk/JSON file containing map data.
2. Generate Rust structs and data from the parsed map.
3. Output the generated code to `target/debug/build/<project_name><your_project_hash>/out/generated_code.rs`.
4. Include the generated code in the main game and use it.

## Dependencies

- `bevy` - A data-driven game engine built in Rust.
- `proc-macro2` - A library for working with Rust's procedural macro API.
- `quote` - A library for generating Rust code.
- `serde, serde_derive, serder_json` - A library for parsing JSON files.
- `once_cell` - A library for lazily-initialized static variables.
