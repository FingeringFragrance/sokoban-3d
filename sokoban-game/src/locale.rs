use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Locale {
    pub lang: String,
}

impl Default for Locale {
    fn default() -> Self {
        Self { lang: "en".to_string() }
    }
}

impl Locale {
    pub fn new(lang: &str) -> Self {
        Self { lang: lang.to_string() }
    }

    #[allow(dead_code)]
    pub fn set_lang(&mut self, lang: &str) {
        self.lang = lang.to_string();
    }

    pub fn lang_label(&self) -> &str {
        match self.lang.as_str() {
            "zh" => "\u{4e2d}\u{6587}",
            "ja" => "\u{65e5}\u{672c}\u{8a9e}",
            _ => "English",
        }
    }

    pub fn t_en(key: &str) -> &str { t_en(key) }
    pub fn t_zh(key: &str) -> &str { t_zh(key) }
}

/// Convenience function to translate a key based on current lang string
pub fn translate<'a>(key: &'a str, lang: &str) -> &'a str {
    match lang {
        "zh" => t_zh(key),
        "ja" => t_ja(key),
        _ => t_en(key),
    }
}

// ---- Menu ----

pub fn t_en(key: &str) -> &str {
    match key {
        "menu.title" => "SOKOBAN 3D",
        "menu.subtitle" => "A 3D Puzzle Adventure",
        "menu.start" => "New Game",
        "menu.continue" => "Continue",
        "menu.settings" => "Settings",
        "menu.exit" => "Exit Game",
        "menu.mode.classic" => "Classic Mode",
        "menu.mode.classic.desc" => "Single-room puzzles. 20 hand-crafted levels.",
        "menu.mode.dungeon" => "Dungeon Mode",
        "menu.mode.dungeon.desc" => "Explore interconnected rooms. Solve puzzles to progress.",
        "menu.mode.daily" => "Daily Challenge",
        "menu.mode.daily.desc" => "Same puzzle for everyone today. Compete for best steps!",
        "menu.mode.multifloor" => "Multi-Floor",
        "menu.mode.multifloor.desc" => "3D puzzles spanning multiple floors. Use stairs & ladders.",
        "menu.back" => "Back",
        "menu.select_level" => "Select Level",
        "menu.grid_nav" => "Arrow keys: Navigate  Enter: Select",
        "menu.not_completed" => "Not completed",
        "settings.title" => "SETTINGS",
        "settings.audio" => "Audio",
        "settings.music_vol" => "Music Volume",
        "settings.sfx_vol" => "SFX Volume",
        "settings.controls" => "Controls",
        "settings.key_move" => "Movement: WASD / Arrow Keys",
        "settings.key_undo" => "Undo: Z",
        "settings.key_restart" => "Restart: R",
        "settings.key_hint" => "Hint: H",
        "settings.key_pause" => "Pause: ESC",
        "settings.camera" => "Camera",
        "settings.camera_mode" => "Camera Mode",
        "settings.camera_free" => "Free Orbit",
        "settings.camera_fixed" => "Fixed Top-Down",
        "settings.camera_follow" => "Follow Player",
        "settings.camera_zoom" => "Zoom",
        "settings.language" => "Language",
        "settings.lang_en" => "English",
        "settings.lang_zh" => "Chinese",
        "settings.lang_ja" => "Japanese",
        "settings.accessibility" => "Accessibility",
        "settings.colorblind" => "Colorblind Mode",
        "settings.high_contrast" => "High Contrast",
        "settings.font_size" => "Font Size",
        "settings.font_small" => "Small",
        "settings.font_normal" => "Normal",
        "settings.font_large" => "Large",
        "hud.level" => "Level",
        "hud.steps" => "Steps",
        "hud.boxes" => "Boxes",
        "hud.on_target" => "on target",
        "hud.room" => "Room",
        "hud.complete" => "Level Complete!",
        "hud.room_complete" => "Room Complete!",
        "hud.press_next" => "Press R to restart, N for next",
        "hud.press_next_room" => "Press N for next room, R to restart",
        "pause.title" => "PAUSED",
        "pause.resume" => "Resume",
        "pause.restart" => "Restart",
        "pause.settings" => "Settings",
        "pause.menu" => "Main Menu",
        "pause.save_exit" => "Save & Exit",
        "tutorial.got_it" => "Press Space or Enter to continue",
        "complete.stars" => "Stars",
        "complete.steps" => "Steps",
        "complete.next" => "Next Level",
        "complete.replay" => "Replay",
        "complete.menu" => "Main Menu",
        _ => key,
    }
}

pub fn t_zh(key: &str) -> &str {
    match key {
        "menu.title" => "\u{63a8}\u{7bb1}\u{5b50} 3D",
        "menu.subtitle" => "3D \u{63a8}\u{7bb1}\u{5b50}\u{89e3}\u{8c1c}\u{5192}\u{9669}",
        "menu.start" => "\u{65b0}\u{6e38}\u{620f}",
        "menu.continue" => "\u{7ee7}\u{7eed}\u{6e38}\u{620f}",
        "menu.settings" => "\u{8bbe}\u{7f6e}",
        "menu.exit" => "\u{9000}\u{51fa}\u{6e38}\u{620f}",
        "menu.mode.classic" => "\u{7ecf}\u{5178}\u{6a21}\u{5f0f}",
        "menu.mode.classic.desc" => "\u{5355}\u{623f}\u{95f4}\u{63a8}\u{7bb1}\u{5b50}\u{3002}20\u{4e2a}\u{7cbe}\u{5fc3}\u{8bbe}\u{8ba1}\u{7684}\u{5173}\u{5361}\u{3002}",
        "menu.mode.dungeon" => "\u{5730}\u{7262}\u{63a2}\u{7d22}",
        "menu.mode.dungeon.desc" => "\u{63a2}\u{7d22}\u{4e92}\u{8054}\u{623f}\u{95f4}\u{3002}\u{89e3}\u{8c1c}\u{524d}\u{8fdb}\u{3002}",
        "menu.mode.daily" => "\u{6bcf}\u{65e5}\u{6311}\u{6218}",
        "menu.mode.daily.desc" => "\u{4eca}\u{5929}\u{5168}\u{7403}\u{73a9}\u{5bb6}\u{5171}\u{540c}\u{6311}\u{6218}\u{3002}\u{6bd4}\u{62fc}\u{6700}\u{4f73}\u{6b65}\u{6570}\u{ff01}",
        "menu.mode.multifloor" => "\u{591a}\u{5c42}\u{6a21}\u{5f0f}",
        "menu.mode.multifloor.desc" => "\u{8de8}\u{8d8a}\u{591a}\u{5c42}\u{7684}\u{7acb}\u{4f53}\u{63a8}\u{7bb1}\u{5b50}\u{3002}\u{4f7f}\u{7528}\u{697c}\u{68af}\u{548c}\u{68af}\u{5b50}\u{3002}",
        "menu.back" => "\u{8fd4}\u{56de}",
        "menu.select_level" => "\u{9009}\u{62e9}\u{5173}\u{5361}",
        "menu.grid_nav" => "\u{65b9}\u{5411}\u{952e}\u{5bfc}\u{822a}  \u{56de}\u{8f66}\u{9009}\u{62e9}",
        "menu.not_completed" => "\u{672a}\u{5b8c}\u{6210}",
        "settings.title" => "\u{8bbe}\u{7f6e}",
        "settings.audio" => "\u{97f3}\u{9891}",
        "settings.music_vol" => "\u{97f3}\u{4e50}\u{97f3}\u{91cf}",
        "settings.sfx_vol" => "\u{97f3}\u{6548}\u{97f3}\u{91cf}",
        "settings.controls" => "\u{63a7}\u{5236}",
        "settings.key_move" => "\u{79fb}\u{52a8}: WASD / \u{65b9}\u{5411}\u{952e}",
        "settings.key_undo" => "\u{64a4}\u{9500}: Z",
        "settings.key_restart" => "\u{91cd}\u{7f6e}: R",
        "settings.key_hint" => "\u{63d0}\u{793a}: H",
        "settings.key_pause" => "\u{6682}\u{505c}: ESC",
        "settings.camera" => "\u{6444}\u{50cf}\u{673a}",
        "settings.camera_mode" => "\u{6444}\u{50cf}\u{673a}\u{6a21}\u{5f0f}",
        "settings.camera_free" => "\u{81ea}\u{7531}\u{8f68}\u{9053}",
        "settings.camera_fixed" => "\u{56fa}\u{5b9a}\u{4fef}\u{89c6}",
        "settings.camera_follow" => "\u{8ddf}\u{968f}\u{73a9}\u{5bb6}",
        "settings.camera_zoom" => "\u{7f29}\u{653e}",
        "settings.language" => "\u{8bed}\u{8a00}",
        "settings.lang_en" => "English",
        "settings.lang_zh" => "\u{4e2d}\u{6587}",
        "settings.lang_ja" => "\u{65e5}\u{672c}\u{8a9e}",
        "settings.accessibility" => "\u{65e0}\u{969c}\u{7887}",
        "settings.colorblind" => "\u{8272}\u{76f2}\u{6a21}\u{5f0f}",
        "settings.high_contrast" => "\u{9ad8}\u{5bf9}\u{6bd4}\u{5ea6}",
        "settings.font_size" => "\u{5b57}\u{4f53}\u{5927}\u{5c0f}",
        "settings.font_small" => "\u{5c0f}",
        "settings.font_normal" => "\u{4e2d}",
        "settings.font_large" => "\u{5927}",
        "hud.level" => "\u{5173}\u{5361}",
        "hud.steps" => "\u{6b65}\u{6570}",
        "hud.boxes" => "\u{7bb1}\u{5b50}",
        "hud.on_target" => "\u{4e2a}\u{5728}\u{76ee}\u{6807}\u{4e0a}",
        "hud.room" => "\u{623f}\u{95f4}",
        "hud.complete" => "\u{901a}\u{5173}\u{ff01}",
        "hud.room_complete" => "\u{623f}\u{95f4}\u{901a}\u{5173}\u{ff01}",
        "hud.press_next" => "\u{6309} R \u{91cd}\u{6765}\u{ff0c}N \u{4e0b}\u{4e00}\u{5173}",
        "hud.press_next_room" => "\u{6309} N \u{4e0b}\u{4e00}\u{623f}\u{95f4}\u{ff0c}R \u{91cd}\u{6765}",
        "pause.title" => "\u{6682}\u{505c}",
        "pause.resume" => "\u{7ee7}\u{7eed}",
        "pause.restart" => "\u{91cd}\u{6765}",
        "pause.settings" => "\u{8bbe}\u{7f6e}",
        "pause.menu" => "\u{4e3b}\u{83dc}\u{5355}",
        "pause.save_exit" => "\u{4fdd}\u{5b58}\u{5e76}\u{9000}\u{51fa}",
        "tutorial.got_it" => "\u{6309}\u{7a7a}\u{683c}\u{6216}\u{56de}\u{8f66}\u{7ee7}\u{7eed}",
        "complete.stars" => "\u{661f}\u{7ea7}",
        "complete.steps" => "\u{6b65}\u{6570}",
        "complete.next" => "\u{4e0b}\u{4e00}\u{5173}",
        "complete.replay" => "\u{91cd}\u{73a9}",
        "complete.menu" => "\u{4e3b}\u{83dc}\u{5355}",
        _ => key,
    }
}

pub fn t_ja(key: &str) -> &str {
    match key {
        "menu.title" => "SOKOBAN 3D",
        "menu.subtitle" => "3D\u{5009}\u{5eab}\u{756a}\u{30d1}\u{30ba}\u{30eb}\u{30a2}\u{30c9}\u{30d9}\u{30f3}\u{30c1}\u{30e3}\u{30fc}",
        "menu.start" => "\u{30cb}\u{30e5}\u{30fc}\u{30b2}\u{30fc}\u{30e0}",
        "menu.continue" => "\u{7d9a}\u{304d}\u{304b}\u{3089}",
        "menu.settings" => "\u{8a2d}\u{5b9a}",
        "menu.exit" => "\u{7d42}\u{4e86}",
        "menu.back" => "\u{623b}\u{308b}",
        "menu.mode.classic" => "\u{30af}\u{30e9}\u{30b7}\u{30c3}\u{30af}",
        "menu.mode.dungeon" => "\u{30c0}\u{30f3}\u{30b8}\u{30e7}\u{30f3}",
        "menu.mode.daily" => "\u{30c7}\u{30a4}\u{30ea}\u{30fc}",
        "menu.mode.multifloor" => "\u{30de}\u{30eb}\u{30c1}\u{30d5}\u{30ed}\u{30a2}",
        "settings.title" => "\u{8a2d}\u{5b9a}",
        "menu.select_level" => "\u{30b9}\u{30c6}\u{30fc}\u{30b8}\u{9078}\u{629e}",
        "menu.grid_nav" => "\u{77e2}\u{5370}\u{30ad}\u{30fc}\u{3067}\u{9078}\u{629e}  Enter\u{3067}\u{6c7a}\u{5b9a}",
        "menu.not_completed" => "\u{672a}\u{30af}\u{30ea}\u{30a2}",
        "pause.title" => "\u{4e00}\u{6642}\u{505c}\u{6b62}",
        "pause.resume" => "\u{518d}\u{958b}",
        "pause.restart" => "\u{3084}\u{308a}\u{76f4}\u{3059}",
        "pause.save_exit" => "\u{4fdd}\u{5b58}\u{3057}\u{3066}\u{7d42}\u{4e86}",
        "pause.settings" => "\u{8a2d}\u{5b9a}",
        "pause.menu" => "\u{30e1}\u{30a4}\u{30f3}\u{30e1}\u{30cb}\u{30e5}\u{30fc}",
        _ => t_en(key),
    }
}
