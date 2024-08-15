use bevy::{
    prelude::*, 
    render::{
        RenderPlugin,
        settings::{
            RenderCreation,
            WgpuSettings,
            Backends
        }
    }
};

const PADDLE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const PADDLE_SIZE: Vec2 = Vec2::new(20.0, 120.0);
const PADDLE_SPEED: f32 = 400.0;

const BALL_COLOR: Color = PADDLE_COLOR;
const BALL_SIZE: Vec2 = Vec2::new(20.0, 20.0);

// TODO:
// 1. ball movement

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, exit_system)
        .add_systems(FixedUpdate, move_paddle)
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: PADDLE_SIZE.extend(1.0),
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle
    ));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(40.0, 0.0, 0.0),
                scale: BALL_SIZE.extend(1.0),
                ..default()
            },
            sprite: Sprite {
                color: BALL_COLOR,
                ..default()
            },
            ..default()
        },
        Ball
    ));
}

fn exit_system(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::KeyQ) {
        info!("User pressed Q key. Exiting...");
        exit.send(AppExit::Success);
    }
}

fn move_paddle(input: Res<ButtonInput<KeyCode>>,
               windows: Query<&Window>,
               mut query: Query<&mut Transform, With<Paddle>>,
               time: Res<Time<Fixed>>) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if input.pressed(KeyCode::KeyW) {
        direction = 1.0;
    }
    
    if input.pressed(KeyCode::KeyS) {
        direction = -1.0;
    }

    let new_paddle_transform = paddle_transform.translation.y + direction * PADDLE_SPEED * time.delta_seconds();
    let window_size = windows.single().size();
    let lower_bound = -window_size.y / 2.0 + PADDLE_SIZE.y / 2.0;
    let upper_bound = window_size.y / 2.0 - PADDLE_SIZE.y / 2.0;

    paddle_transform.translation.y = new_paddle_transform.clamp(lower_bound, upper_bound);
}
