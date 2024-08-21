use std::time::Duration;
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

const SCORING_ZONE_SIZE: Vec2 = BALL_SIZE;
const LEFT_SCORING_ZONE_LOCATION: Vec2 = Vec2::new(-WINDOW_SIZE.x / 2.0 - SCORING_ZONE_SIZE.x / 2.0, 0.0);
const RIGHT_SCORING_ZONE_LOCATION: Vec2 = Vec2::new(WINDOW_SIZE.x / 2.0 + SCORING_ZONE_SIZE.x / 2.0, 0.0);

const SCORE_UI_PADDING: Val = Val::Px(5.0);
const SCORE_UI_COLOR: Color = Color::WHITE;
const SCORE_UI_FONT_SIZE: f32 = 40.0;

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
        .init_state::<GameState>()
        .insert_resource(Score { left: 0, right: 0 })
        .add_systems(Startup, setup)
        .add_systems(Update,
            (
                exit_system,
                update_score_ui
            )
        )
        .add_systems(FixedUpdate,
            move_player_paddle
        )
        .add_systems(FixedUpdate,
            (
                apply_velocity,
                handle_collisions
            )
            .run_if(in_state(GameState::Playing))
            .chain(),
        )
        .add_systems(OnEnter(GameState::Respawning), start_ball_respawn_timer)
        .add_systems(Update,
            (
                tick_ball_respawn_timer
            )
            .run_if(in_state(GameState::Respawning))
        )
        .run();
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Playing,
    Respawning,
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

enum Side {
    Left,
    Right
}

#[derive(Component)]
struct ScoringZone {
    side: Side
}

#[derive(Component)]
struct ScoreUi(Side);

#[derive(Resource)]
struct Score {
    left: usize,
    right: usize
}

#[derive(Bundle)]
struct PaddleBundle {
    sprite_bundle: SpriteBundle,
    paddle: Paddle,
    collider: Collider,
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
            collider: Collider,
        }
    }
}

#[derive(Bundle)]
struct ScoringZoneBundle {
    transform_bundle: TransformBundle,
    scoring_zone: ScoringZone,
    collider:Collider
}

impl ScoringZoneBundle {
    fn new(starting_location: Vec2, side: Side) -> Self {
        Self {
            transform_bundle: TransformBundle {
                local: Transform {
                    translation: starting_location.extend(1.0),
                    scale: SCORING_ZONE_SIZE.extend(1.0),
                    ..default()
                },
                ..default()
            },
            scoring_zone: ScoringZone { side },
            collider: Collider
        }
    }
}

#[derive(Bundle)]
struct ScoreUiBundle {
    text_bundle: TextBundle,
    score_ui: ScoreUi,
}

impl ScoreUiBundle {
    fn new(side: Side) -> Self {
        let style = match side {
            Side::Left => Style {
                position_type: PositionType::Absolute,
                top: SCORE_UI_PADDING,
                left: SCORE_UI_PADDING,
                ..default()
            },
            Side::Right => Style {
                position_type: PositionType::Absolute,
                top: SCORE_UI_PADDING,
                right: SCORE_UI_PADDING,
                ..default()
            }
        };

        Self {
            text_bundle: TextBundle::from_sections([
                TextSection::new(
                    "Score: ",
                    TextStyle {
                        font_size: SCORE_UI_FONT_SIZE,
                        color: SCORE_UI_COLOR,
                        ..default()
                    }
                ),
                TextSection::from_style(TextStyle {
                    font_size: SCORE_UI_FONT_SIZE,
                    color: SCORE_UI_COLOR,
                    ..default()
                }),
            ]).with_style(style),
            score_ui: ScoreUi(side),
        }
    }
}

#[derive(Bundle)]
struct BallBundle {
    sprite_bundle: SpriteBundle,
    ball: Ball,
    velocity: Velocity,
}

impl BallBundle {
    fn new(location: Vec2, direction: Vec2) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.extend(1.0),
                    scale: BALL_SIZE.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: BALL_COLOR,
                    ..default()
                },
                ..default()
            },
            ball: Ball,
            velocity: Velocity(direction.normalize() * BALL_SPEED)
        }
    }
}

#[derive(Component)]
struct RespawnTimer(Timer);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((PaddleBundle::new(PLAYER_PADDLE_START_LOCATION), Player));
    commands.spawn(PaddleBundle::new(OPPONENT_PADDLE_START_LOCATION));
    commands.spawn(BallBundle::new(BALL_START_LOCATION, BALL_START_DIRECTION));
    commands.spawn(ScoringZoneBundle::new(LEFT_SCORING_ZONE_LOCATION, Side::Left));
    commands.spawn(ScoringZoneBundle::new(RIGHT_SCORING_ZONE_LOCATION, Side::Right));
    commands.spawn(ScoreUiBundle::new(Side::Left));
    commands.spawn(ScoreUiBundle::new(Side::Right));
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
    info!("{}", time.delta_seconds());
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn handle_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut ball_query: Query<(Entity, &mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(&Transform, Option<&ScoringZone>), With<Collider>>,
    mut next_state: ResMut<NextState<GameState>>
) {
    if ball_query.is_empty() {
        // The ball has been despawned
        // TODO - maybe we could avoid this check by using states?
        return;
    }

    let (ball_entity, mut ball_velocity, ball_transform) = ball_query.single_mut();

    for (collider_transform, maybe_scoring_zone) in &collider_query {
        let ball_bb = Aabb2d::new(ball_transform.translation.truncate(), ball_transform.scale.truncate() / 2.0);
        let collider_bb = Aabb2d::new(collider_transform.translation.truncate(), collider_transform.scale.truncate() / 2.0);

        if ball_bb.intersects(&collider_bb) {
            if let Some(scoring_zone) = maybe_scoring_zone {
                commands.entity(ball_entity).despawn();
                next_state.set(GameState::Respawning);
                match scoring_zone.side {
                    Side::Left => score.left += 1,
                    Side::Right => score.right += 1,
                }
            } else {
                // reflect x
                ball_velocity.x = -ball_velocity.x
            }
        }
    }
}

fn update_score_ui(score: ResMut<Score>, mut query: Query<(&mut Text, &ScoreUi)>) {
    for (mut text, ScoreUi(side)) in &mut query {
        match side {
            Side::Left => text.sections[1].value = score.left.to_string(),
            Side::Right => text.sections[1].value = score.right.to_string(),
        }
    }
}

fn start_ball_respawn_timer(mut commands: Commands) {
    info!("Starting respawn timer...");
    commands.spawn(RespawnTimer(Timer::new(Duration::from_secs(3), TimerMode::Once)));
}

fn tick_ball_respawn_timer(
    mut commands: Commands,
    mut query: Query<&mut RespawnTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>
) {
    let mut respawn_timer = query.single_mut();
    respawn_timer.0.tick(time.delta());

    if respawn_timer.0.finished() {
        info!("Respawning ball...");
        commands.spawn(BallBundle::new(BALL_START_LOCATION, BALL_START_LOCATION));
        next_state.set(GameState::Playing);
    }
}
