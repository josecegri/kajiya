use anyhow::{Context, anyhow};
use kajiya_simple::{
    KeyMap, KeyboardMap,
    winit::keyboard::{KeyCode, PhysicalKey},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, canonicalize},
    io::Read,
    path::PathBuf,
};
use toml::from_str;

#[derive(Serialize, Deserialize, Default, Clone)]
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

        // TODO restore keymap.toml file parsing
        // Don't use anyhow context here because it doesn't show the parsing error.
        let keymap = from_str(&buffer)
            .map_err(|e| anyhow!("Failed to parse keymap.toml: {}", e.to_string()))?;

        Ok(keymap)
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Ui {
    pub toggle: PhysicalKey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sequencer {
    pub add_keyframe: PhysicalKey,
    pub play: PhysicalKey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Rendering {
    pub switch_to_reference_path_tracing: PhysicalKey,
    pub reset_path_tracer: PhysicalKey,
    pub light_enable_emissive: PhysicalKey,
}

#[derive(Serialize, Deserialize, Clone)]
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
