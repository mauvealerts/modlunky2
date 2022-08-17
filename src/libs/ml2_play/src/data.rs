use ml2_net::http::DownloadProgress;
use serde::{Deserialize, Serialize};

const DEFAULT_FONT: &str = "default";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    Local(String),
    Nightly,
    Stable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Waiting,
    Starting,
    Downloading(DownloadProgress),
    Running,
    Finished,
}

pub type LoadOrderConfig = Vec<LoadMod>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadMod {
    pub enabled: bool,
    pub id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaylunkyConfig {
    general_settings: GeneralSettings,
    script_settings: ScriptSettings,
    audio_settings: AudioSettings,
    sprite_settings: SpriteSettings,
    bug_fixes: BugFixes,
    key_bindings: KeyBindings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    enable_loose_file_warning: bool,
    enable_raw_string_loading: bool,
    disable_asset_caching: bool,
    block_save_game: bool,
    allow_save_game_mods: bool,
    use_playlunky_save: bool,
    disable_steam_achievements: bool,
    speedrun_mode: bool,
    font_file: String,
    font_file_ru: String,
    font_file_jp: String,
    font_file_ko: String,
    font_file_zhcn: String,
    font_file_zhtw: String,
    font_file_emoji: String,
    font_scale: f32,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            enable_loose_file_warning: true,
            enable_raw_string_loading: false,
            disable_asset_caching: false,
            block_save_game: false,
            allow_save_game_mods: true,
            use_playlunky_save: false,
            disable_steam_achievements: false,
            speedrun_mode: false,
            font_file: DEFAULT_FONT.to_string(),
            font_file_ru: DEFAULT_FONT.to_string(),
            font_file_jp: DEFAULT_FONT.to_string(),
            font_file_ko: DEFAULT_FONT.to_string(),
            font_file_zhcn: DEFAULT_FONT.to_string(),
            font_file_zhtw: DEFAULT_FONT.to_string(),
            font_file_emoji: DEFAULT_FONT.to_string(),
            font_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScriptSettings {
    enable_developer_mode: bool,
    enable_developer_console: bool,
    console_history_size: i32,
}

impl Default for ScriptSettings {
    fn default() -> Self {
        Self {
            enable_developer_mode: false,
            enable_developer_console: false,
            console_history_size: 20,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    enable_loose_audio_files: bool,
    cache_decoded_audio_files: bool,
    synchronous_update: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enable_loose_audio_files: true,
            cache_decoded_audio_files: false,
            synchronous_update: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpriteSettings {
    random_character_select: bool,
    link_related_files: bool,
    generate_character_journal_stickers: bool,
    generate_character_journal_entries: bool,
    generate_sticker_pixel_art: bool,
    enable_sprite_hot_loading: bool,
    sprite_hot_load_delay: i32,
    enable_customizable_sheets: bool,
    enable_luminance_scaling: bool,
}

impl Default for SpriteSettings {
    fn default() -> Self {
        Self {
            random_character_select: false,
            link_related_files: true,
            generate_character_journal_stickers: true,
            generate_character_journal_entries: true,
            generate_sticker_pixel_art: true,
            enable_sprite_hot_loading: false,
            sprite_hot_load_delay: 400,
            enable_customizable_sheets: true,
            enable_luminance_scaling: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BugFixes {
    out_of_bounds_liquids: bool,
    missing_thorns: bool,
    missing_pipes: bool,
}

impl Default for BugFixes {
    fn default() -> Self {
        Self {
            out_of_bounds_liquids: true,
            missing_thorns: true,
            missing_pipes: false,
        }
    }
}

// We can (de)serialize this directly since everything's a string
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindings {
    console: u64,
    console_alt: u64,
    console_close: u64,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            console: 0xc0,
            console_alt: 0xdc,
            console_close: 0x1b,
        }
    }
}
