use bevy::audio::{AudioPlayer, AudioSource, PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameAudio {
    pub move_sound: Handle<AudioSource>,
    pub push_sound: Handle<AudioSource>,
    pub target_sound: Handle<AudioSource>,
    pub complete_sound: Handle<AudioSource>,
    pub collect_sound: Handle<AudioSource>,
    pub door_sound: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct GameVolume(pub f32);

impl Default for GameVolume {
    fn default() -> Self {
        Self(0.7)
    }
}

impl GameVolume {
    pub fn increase(&mut self) {
        self.0 = (self.0 + 0.1).min(1.0);
    }

    pub fn decrease(&mut self) {
        self.0 = (self.0 - 0.1).max(0.0);
    }

    pub fn label(&self) -> String {
        format!("{}%", (self.0 * 100.0).round() as u32)
    }
}

/// 加载音频资源（支持 .wav 和 .ogg，缺失时游戏正常运行但无声）
pub fn load_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sounds_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/audio/sounds");
    if !std::path::Path::new(sounds_dir).exists() {
        println!("Audio: assets/audio/sounds/ not found, running without sound");
        return;
    }

    // 检测文件格式（优先 .wav，回退 .ogg）
    let ext = if std::path::Path::new(&format!("{}/move.ogg", sounds_dir)).exists() {
        "ogg"
    } else if std::path::Path::new(&format!("{}/move.wav", sounds_dir)).exists() {
        "wav"
    } else {
        println!("Audio: no sound files found, running without sound");
        return;
    };

    let path = |name: &str| -> String {
        format!("audio/sounds/{}.{}", name, ext)
    };

    commands.insert_resource(GameAudio {
        move_sound: asset_server.load(path("move")),
        push_sound: asset_server.load(path("push")),
        target_sound: asset_server.load(path("target")),
        complete_sound: asset_server.load(path("complete")),
        collect_sound: asset_server.load(path("collect")),
        door_sound: asset_server.load(path("door")),
    });

    println!("Audio: loaded {} sound files ({})", 6, ext);
}

/// 播放一次音效
pub fn play_sound(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            speed: 1.0,
            paused: false,
            ..default()
        },
    ));
}

#[derive(Resource)]
pub struct BgmState {
    pub current_theme: String,
    pub bgm_handles: BgmHandles,
    pub loaded: bool,
}

#[derive(Clone)]
pub struct BgmHandles {
    pub default_bgm: Handle<AudioSource>,
    pub forest_bgm: Handle<AudioSource>,
    pub volcano_bgm: Handle<AudioSource>,
    pub ice_palace_bgm: Handle<AudioSource>,
    pub sky_temple_bgm: Handle<AudioSource>,
    pub ruins_bgm: Handle<AudioSource>,
    pub void_bgm: Handle<AudioSource>,
}

impl Default for BgmState {
    fn default() -> Self {
        Self {
            current_theme: String::new(),
            bgm_handles: BgmHandles {
                default_bgm: Handle::default(),
                forest_bgm: Handle::default(),
                volcano_bgm: Handle::default(),
                ice_palace_bgm: Handle::default(),
                sky_temple_bgm: Handle::default(),
                ruins_bgm: Handle::default(),
                void_bgm: Handle::default(),
            },
            loaded: false,
        }
    }
}

pub fn load_bgm(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/audio/music");
    if !std::path::Path::new(music_dir).exists() {
        println!("BGM: assets/audio/music/ not found, running without BGM");
        return;
    }

    let ext = if std::path::Path::new(&format!("{}/default.ogg", music_dir)).exists() {
        "ogg"
    } else if std::path::Path::new(&format!("{}/default.wav", music_dir)).exists() {
        "wav"
    } else {
        println!("BGM: no music files found, running without BGM");
        return;
    };

    let path = |name: &str| -> String {
        format!("audio/music/{}.{}", name, ext)
    };

    commands.insert_resource(BgmState {
        current_theme: String::new(),
        bgm_handles: BgmHandles {
            default_bgm: asset_server.load(path("default")),
            forest_bgm: asset_server.load(path("forest")),
            volcano_bgm: asset_server.load(path("volcano")),
            ice_palace_bgm: asset_server.load(path("ice_palace")),
            sky_temple_bgm: asset_server.load(path("sky_temple")),
            ruins_bgm: asset_server.load(path("ruins")),
            void_bgm: asset_server.load(path("void")),
        },
        loaded: true,
    });

    println!("BGM: loaded music files ({})", ext);
}

pub fn play_bgm(commands: &mut Commands, bgm_state: &BgmState, theme: &str, volume: f32) {
    if !bgm_state.loaded {
        return;
    }

    let handle = match theme {
        "forest" => &bgm_state.bgm_handles.forest_bgm,
        "volcano" => &bgm_state.bgm_handles.volcano_bgm,
        "ice_palace" => &bgm_state.bgm_handles.ice_palace_bgm,
        "sky_temple" => &bgm_state.bgm_handles.sky_temple_bgm,
        "ruins" => &bgm_state.bgm_handles.ruins_bgm,
        "void" => &bgm_state.bgm_handles.void_bgm,
        _ => &bgm_state.bgm_handles.default_bgm,
    };

    commands.spawn((
        AudioPlayer(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(volume * 0.5),
            speed: 1.0,
            paused: false,
            ..default()
        },
    ));
}

pub fn bgm_switch_system(
    mut commands: Commands,
    mut bgm_state: ResMut<BgmState>,
    game_state: Option<Res<crate::game::GameState>>,
    volume: Res<crate::audio::GameVolume>,
    bgm_entities: Query<Entity, With<BgmMarker>>,
) {
    let Some(ref gs) = game_state else { return };

    if gs.scene_theme == bgm_state.current_theme {
        return;
    }

    for entity in &bgm_entities {
        commands.entity(entity).despawn();
    }

    bgm_state.current_theme = gs.scene_theme.clone();
    play_bgm(&mut commands, &bgm_state, &gs.scene_theme, volume.0);
}

#[derive(Component)]
pub struct BgmMarker;
