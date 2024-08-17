use bevy::{
    math::bounding::{
        Aabb2d,
        IntersectsVolume
    },
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

const WINDOW_SIZE: Vec2 = Vec2::new(800.0, 600.0);

const PADDLE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const PADDLE_SIZE: Vec2 = Vec2::new(20.0, 120.0);
const PADDLE_SPEED: f32 = 400.0;
const PLAYER_PADDLE_START_LOCATION: Vec2 = Vec2::new(-WINDOW_SIZE.x / 2.0 + PADDLE_SIZE.x + PADDLE_SIZE.x / 2.0, 0.0);
const OPPONENT_PADDLE_START_LOCATION: Vec2 = Vec2::new(WINDOW_SIZE.x / 2.0 - PADDLE_SIZE.x - PADDLE_SIZE.x / 2.0, 0.0);

const BALL_COLOR: Color = PADDLE_COLOR;
const BALL_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const BALL_SPEED: f32 = 500.0;
const BALL_START_LOCATION: Vec2 = Vec2::new(0.0, 0.0);
const BALL_START_DIRECTION: Vec2 = Vec2::new(-1.0, 0.0);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pong".into(),
                        resolution: WINDOW_SIZE.into(),
                        ..default()
                    }),
                    ..default()
                })
            )
        .add_systems(Startup, setup)
        .add_systems(Update, exit_system)
        .add_systems(FixedUpdate,
            (
                move_player_paddle,
                apply_velocity,
                handle_collisions
            ).chain()
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Collider;

// Deref allows the Vec2 to be access directly instead of velocity.0.x
#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Bundle)]
struct PaddleBundle {
    sprite_bundle: SpriteBundle,
    paddle: Paddle,
    colldier: Collider,
}

impl PaddleBundle {
    fn new(start_location: Vec2) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: start_location.extend(1.0),
                    scale: PADDLE_SIZE.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: PADDLE_COLOR,
                    ..default()
                },
                ..default()
            },
            paddle: Paddle,
            colldier: Collider,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((PaddleBundle::new(PLAYER_PADDLE_START_LOCATION), Player));
    commands.spawn(PaddleBundle::new(OPPONENT_PADDLE_START_LOCATION));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: BALL_START_LOCATION.extend(1.0),
                scale: BALL_SIZE.extend(1.0),
                ..default()
            },
            sprite: Sprite {
                color: BALL_COLOR,
                ..default()
            },
            ..default()
        },
        Ball,
        Velocity(BALL_START_DIRECTION.normalize() * BALL_SPEED)
    ));
}

fn exit_system(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::KeyQ) {
        info!("User pressed Q key. Exiting...");
        exit.send(AppExit::Success);
    }
}

fn move_player_paddle(
    input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut query: Query<&mut Transform, (With<Player>, With<Paddle>)>,
    time: Res<Time<Fixed>>
) {
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

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time<Fixed>>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn handle_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<&Transform, With<Collider>>
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    for collider_transform in &collider_query {
        let ball_bb = Aabb2d::new(ball_transform.translation.truncate(), ball_transform.scale.truncate() / 2.0);
        let collider_bb = Aabb2d::new(collider_transform.translation.truncate(), collider_transform.scale.truncate() / 2.0);

        if ball_bb.intersects(&collider_bb) {
            // reflect x
            ball_velocity.x = -ball_velocity.x
        }
    }
}
