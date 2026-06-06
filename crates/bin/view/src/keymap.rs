use anyhow::{Context, anyhow};
use kajiya_simple::{
    KeyMap, KeyboardMap,
    winit::keyboard::{KeyCode, PhysicalKey},
};
use serde::Deserialize;
use std::{
    fs::{File, canonicalize},
    io::Read,
    path::PathBuf,
};
use toml::from_str;

#[derive(Default, Clone)]
pub struct KeymapConfig {
    pub movement: Movement,
    pub ui: Ui,
    pub sequencer: Sequencer,
    pub rendering: Rendering,
    pub misc: Misc,
}

impl KeymapConfig {
    pub(crate) fn load(path: &Option<PathBuf>) -> anyhow::Result<Self> {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = path.clone().unwrap_or(base.join("keymap.toml"));
        let path = canonicalize(path).with_context(|| {
            "Failed to find keymap.toml. Make sure it is in the same directory as the executable."
        })?;

        let mut keymap_file = File::open(path).with_context(|| "Failed to open keymap.toml")?;

        let mut buffer = String::new();
        keymap_file
            .read_to_string(&mut buffer)
            .with_context(|| "Failed to read keymap.toml")?;

        // Don't use anyhow context here because it doesn't show the parsing error.
        let keymap: KeymapConfigToml = from_str(&buffer)
            .map_err(|e| anyhow!("Failed to parse keymap.toml: {}", e.to_string()))?;
        Ok(keymap.into())
    }
}

impl From<Movement> for KeyboardMap {
    fn from(val: Movement) -> Self {
        KeyboardMap::new()
            .bind(val.forward, KeyMap::new("move_fwd", 1.0))
            .bind(val.backward, KeyMap::new("move_fwd", -1.0))
            .bind(val.right, KeyMap::new("move_right", 1.0))
            .bind(val.left, KeyMap::new("move_right", -1.0))
            .bind(val.up, KeyMap::new("move_up", 1.0))
            .bind(val.down, KeyMap::new("move_up", -1.0))
            .bind(val.boost, KeyMap::new("boost", 1.0).activation_time(0.25))
            .bind(val.slow, KeyMap::new("boost", -1.0).activation_time(0.5))
    }
}

#[derive(Clone)]
pub struct Movement {
    forward: PhysicalKey,
    backward: PhysicalKey,
    left: PhysicalKey,
    right: PhysicalKey,
    up: PhysicalKey,
    down: PhysicalKey,
    boost: PhysicalKey,
    slow: PhysicalKey,
}

#[derive(Clone)]
pub struct Ui {
    pub toggle: PhysicalKey,
}

#[derive(Clone)]
pub struct Sequencer {
    pub add_keyframe: PhysicalKey,
    pub play: PhysicalKey,
}

#[derive(Clone)]
pub struct Rendering {
    pub switch_to_reference_path_tracing: PhysicalKey,
    pub reset_path_tracer: PhysicalKey,
    pub light_enable_emissive: PhysicalKey,
}

#[derive(Clone)]
pub struct Misc {
    pub print_camera_transform: PhysicalKey,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            forward: PhysicalKey::Code(KeyCode::KeyW),
            backward: PhysicalKey::Code(KeyCode::KeyS),
            left: PhysicalKey::Code(KeyCode::KeyA),
            right: PhysicalKey::Code(KeyCode::KeyD),
            up: PhysicalKey::Code(KeyCode::KeyE),
            down: PhysicalKey::Code(KeyCode::KeyQ),
            boost: PhysicalKey::Code(KeyCode::ShiftLeft),
            slow: PhysicalKey::Code(KeyCode::ControlLeft),
        }
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            toggle: PhysicalKey::Code(KeyCode::Tab),
        }
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self {
            add_keyframe: PhysicalKey::Code(KeyCode::KeyK),
            play: PhysicalKey::Code(KeyCode::KeyP),
        }
    }
}

impl Default for Rendering {
    fn default() -> Self {
        Self {
            switch_to_reference_path_tracing: PhysicalKey::Code(KeyCode::Space),
            reset_path_tracer: PhysicalKey::Code(KeyCode::Backspace),
            light_enable_emissive: PhysicalKey::Code(KeyCode::KeyL),
        }
    }
}

impl Default for Misc {
    fn default() -> Self {
        Self {
            print_camera_transform: PhysicalKey::Code(KeyCode::KeyC),
        }
    }
}

// keymap deserialization plumbing

#[derive(Deserialize, Clone)]
struct KeymapConfigToml {
    movement: MovementConfigToml,
    ui: UiConfigToml,
    sequencer: SequencerConfigToml,
    rendering: RenderingConfigToml,
    misc: MiscConfigToml,
}

#[derive(Deserialize, Clone)]
struct MovementConfigToml {
    forward: ConfigKey,
    backward: ConfigKey,
    left: ConfigKey,
    right: ConfigKey,
    up: ConfigKey,
    down: ConfigKey,
    boost: ConfigKey,
    slow: ConfigKey,
}

#[derive(Deserialize, Clone)]
struct UiConfigToml {
    toggle: ConfigKey,
}

#[derive(Deserialize, Clone)]
struct SequencerConfigToml {
    add_keyframe: ConfigKey,
    play: ConfigKey,
}

#[derive(Deserialize, Clone)]
struct RenderingConfigToml {
    switch_to_reference_path_tracing: ConfigKey,
    reset_path_tracer: ConfigKey,
    light_enable_emissive: ConfigKey,
}

#[derive(Deserialize, Clone)]
struct MiscConfigToml {
    print_camera_transform: ConfigKey,
}

impl From<KeymapConfigToml> for KeymapConfig {
    fn from(value: KeymapConfigToml) -> Self {
        Self {
            movement: value.movement.into(),
            ui: value.ui.into(),
            sequencer: value.sequencer.into(),
            rendering: value.rendering.into(),
            misc: value.misc.into(),
        }
    }
}

impl From<MovementConfigToml> for Movement {
    fn from(value: MovementConfigToml) -> Self {
        Self {
            forward: value.forward.0,
            backward: value.backward.0,
            left: value.left.0,
            right: value.right.0,
            up: value.up.0,
            down: value.down.0,
            boost: value.boost.0,
            slow: value.slow.0,
        }
    }
}

impl From<UiConfigToml> for Ui {
    fn from(value: UiConfigToml) -> Self {
        Self {
            toggle: value.toggle.0,
        }
    }
}

impl From<SequencerConfigToml> for Sequencer {
    fn from(value: SequencerConfigToml) -> Self {
        Self {
            add_keyframe: value.add_keyframe.0,
            play: value.play.0,
        }
    }
}

impl From<RenderingConfigToml> for Rendering {
    fn from(value: RenderingConfigToml) -> Self {
        Self {
            switch_to_reference_path_tracing: value.switch_to_reference_path_tracing.0,
            reset_path_tracer: value.reset_path_tracer.0,
            light_enable_emissive: value.light_enable_emissive.0,
        }
    }
}

impl From<MiscConfigToml> for Misc {
    fn from(value: MiscConfigToml) -> Self {
        Self {
            print_camera_transform: value.print_camera_transform.0,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(transparent)]
struct ConfigKey(#[serde(deserialize_with = "deserialize_key")] PhysicalKey);

fn deserialize_key<'de, D>(deserializer: D) -> Result<PhysicalKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let key_name = match s.as_bytes() {
        [c] if c.is_ascii_alphabetic() => {
            format!("Key{}", s.to_ascii_uppercase())
        }
        _ => s,
    };

    let keycode: KeyCode = serde_plain::from_str(&key_name).map_err(serde::de::Error::custom)?;

    Ok(PhysicalKey::Code(keycode))
}
