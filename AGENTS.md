# AGENTS.md - Development Guidelines for Monkey 3D Game

## Build Commands
- `cargo build` - Build the project
- `cargo run` - Run the game locally  
- `cargo run --target wasm32-unknown-unknown` - Build for web
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run single test
- `cargo check` - Quick syntax/type checking
- `cargo clippy` - Lint with clippy

## Code Style Guidelines

### Imports & Organization
- Group imports: std crates → external crates → local modules
- Use `use bevy::prelude::*` for Bevy components
- Local imports use full path: `crate::utils::module::item`

### Naming Conventions
- Constants: `SCREAMING_SNAKE_CASE` in dedicated modules
- Components: `PascalCase` with `#[derive(Component)]`
- Resources: `PascalCase` with `#[derive(Resource)]` 
- Functions/variables: `snake_case`
- Enums: `PascalCase` with descriptive variants

### Types & Patterns
- Use `#[derive(Default)]` where appropriate
- Prefer `Option<T>` over nullable values
- Use `Result<T, E>` for error handling
- Mark game entities with `GameEntity` component
- Mark UI entities with `UIEntity` component

### Bevy Specific
- Use plugins for major systems (`impl Plugin`)
- Systems in `Startup` schedule for initialization
- Use `ResMut`/`Res` for resource access
- Components should be small and focused
- Use `Time<Fixed>` for physics/timestep logic

### File Structure
- Each major system in separate module under `utils/`
- Constants organized in submodules by feature
- Main game logic in `game_functions.rs`
- Keep `main.rs` focused on app setup only