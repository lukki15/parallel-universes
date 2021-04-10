use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::pass::ClearColor;

const SCORE_BOARD_HEIGHT: u32 = 2;
const ARENA_HEIGHT: u32 = 16;
const ARENA_WIDTH: u32 = 24;

const SPRITE_HEIGHT: u32 = 32;
const SPRITE_WIDTH: u32 = 32;

const WINDOW_HEIGHT: u32 = (ARENA_HEIGHT + SCORE_BOARD_HEIGHT) * SPRITE_HEIGHT;
const WINDOW_WIDTH: u32 = ARENA_WIDTH * SPRITE_WIDTH;

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SnakeMovement {
    Input,
    Movement,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct Rocket {
    direction: Direction,
}

struct Wall {}
struct Target {}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
    StandStill,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::StandStill => Self::StandStill,
        }
    }
}

fn spawn_wall(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    wall_color: Color,
    wall_position: Position,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(wall_color.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Wall {})
        .insert(wall_position)
        .insert(Size::square(0.9));
}

fn spawn_target(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    wall_position: Position,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.8, 0.0).into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Target {})
        .insert(wall_position)
        .insert(Size::square(0.9));
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let level_data: Vec<String> = vec![
        "          WWWW        ".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW    T   ".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW          WWWWW".to_string(),
        "   WWWW          WWWWW".to_string(),
        "   WWWW          WWWWW".to_string(),
        "s  WWWW          WWWWW".to_string(),
    ];

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Add border walls
    let wall_color = Color::rgb(0.8, 0.8, 0.8);
    for y in 0..ARENA_HEIGHT as i32 {
        spawn_wall(
            &mut commands,
            &mut materials,
            wall_color,
            Position { x: 0, y },
        );
        spawn_wall(
            &mut commands,
            &mut materials,
            wall_color,
            Position {
                x: ARENA_WIDTH as i32 - 1,
                y,
            },
        );
    }
    for x in 1..ARENA_WIDTH as i32 - 1 {
        spawn_wall(
            &mut commands,
            &mut materials,
            wall_color,
            Position { x, y: 0 },
        );
        spawn_wall(
            &mut commands,
            &mut materials,
            wall_color,
            Position {
                x,
                y: ARENA_HEIGHT as i32 - 1,
            },
        );
    }

    for (y, line_data) in level_data.iter().rev().enumerate() {
        for (x, c) in line_data.chars().enumerate() {
            let pos = Position {
                x: x as i32 + 1,
                y: y as i32 + 1,
            };
            if c == 'W' {
                spawn_wall(&mut commands, &mut materials, wall_color, pos);
            } else if c == 'T' {
                spawn_target(&mut commands, &mut materials, pos);
            }
        }
    }
}

fn spawn_rocket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("icon.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Rocket {
            direction: Direction::StandStill,
        })
        .insert(Position { x: 1, y: 1 })
        .insert(Size::square(0.8))
        .id();
}

fn rocket_movement_input(keyboard_input: Res<Input<KeyCode>>, mut rockets: Query<&mut Rocket>) {
    let right = keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D);
    let left = keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A);
    let up = keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W);
    let down = keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S);

    if let Some(mut rocket) = rockets.iter_mut().next() {
        let dir: Direction = if left {
            Direction::Left
        } else if down {
            Direction::Down
        } else if up {
            Direction::Up
        } else if right {
            Direction::Right
        } else {
            rocket.direction
        };
        if dir != rocket.direction.opposite() {
            rocket.direction = dir;
        }
    }
}
// Query<(&mut Position, &Rocket)>, Query<&Position, With<Wall>>
fn rocket_movement(
    mut rocket_query: Query<(&Rocket, &mut Position)>,
    collider_query: Query<&Transform, With<Wall>>,
    windows: Res<Windows>
) {
    let window = windows.get_primary().unwrap();
    if let Some((rocket, mut rocket_pos)) = rocket_query.iter_mut().next() {
        let mut next_position = rocket_pos.clone();

        match &rocket.direction {
            Direction::Left => {
                if rocket_pos.x > 0 {
                    next_position.x -= 1;
                }
            }
            Direction::Right => {
                if rocket_pos.x < ARENA_WIDTH as i32 - 1 {
                    next_position.x += 1;
                }
            }
            Direction::Up => {
                if rocket_pos.y < ARENA_HEIGHT as i32 - 1 {
                    next_position.y += 1;
                }
            }
            Direction::Down => {
                if rocket_pos.y > 0 {
                    next_position.y -= 1;
                }
            }
            Direction::StandStill => {}
        };

        let mut no_collision = true;
        
        for wall_transform in collider_query.iter(){
            
            let next_transform_x = convert_x(next_position.x, window.width());
            let next_transform_y = convert_y(
                next_position.y ,
                window.height()
            );
            if wall_transform.translation.x == next_transform_x && wall_transform.translation.y == next_transform_y{
                no_collision = false;
            }
        }
        
        if no_collision {
            rocket_pos.x = next_position.x;
            rocket_pos.y = next_position.y;
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / (ARENA_HEIGHT + SCORE_BOARD_HEIGHT) as f32
                * window.height() as f32,
        );
    }
}

fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
    let tile_size = bound_window / bound_game;
    pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
}

fn convert_x(pos: i32, window_width: f32) -> f32{
    convert(pos as f32, window_width, ARENA_WIDTH as f32)
}

fn convert_y(pos: i32, window_height: f32) -> f32{
    convert(
        pos as f32,
        window_height,
        (ARENA_HEIGHT + SCORE_BOARD_HEIGHT) as f32,
    )
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert_x(pos.x, window.width()),
            convert_y(
                pos.y,
                window.height()
            ),
            0.0,
        );
    }
}

fn main() {
    let mut app = App::build();
    app.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "PARALLEL UNIVERSES".to_string(),
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            ..Default::default()
        })
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_rocket.system()))
        .add_system(
            rocket_movement_input
                .system()
                .label(SnakeMovement::Input)
                .before(SnakeMovement::Movement),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.050))
                .with_system(rocket_movement.system().label(SnakeMovement::Movement)),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation.system())
                .with_system(size_scaling.system()),
        )
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.run();
}
