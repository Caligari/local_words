use eframe::egui::{Color32, Vec2};

pub const UI_PADDING: f32 = 8.0;
pub const ERROR_SPACE: f32 = 16.0;

pub const ERROR_BACKGROUND: Color32 = Color32::from_rgb(255, 190, 190);
pub const ERROR_FOREGROUND: Color32 = Color32::DARK_RED;

pub const MODE_COLOR: Color32 = Color32::DARK_GREEN;

// layout constants
pub const EDGE_COLUMN_WIDTH: f32 = 40.0;
pub const INDENT_COLUMN_WIDTH: f32 = 16.0;
pub const BETWEEN_FIELDS: f32 = 8.0;
pub const TINY_SPACE: f32 = 2.0;
pub const SMALL_SPACE: f32 = 5.0;
pub const BETWEEN_COLS: f32 = 12.0;
pub const STRING_ROWS: usize = 8; // depends on font size, surely
pub const STRING_WIDTH: f32 = 500.0;
pub const STRING_HEIGHT: f32 = 200.0;
pub const STRING_RECT: Vec2 = Vec2 {
    x: STRING_WIDTH,
    y: STRING_HEIGHT,
};

pub const ACTIVE_COLOR: Color32 = Color32::DARK_GREEN; // should change with theme
pub const MISSING_COLOR: Color32 = Color32::RED;
pub const MOD_MAIN_COLOR: Color32 = Color32::GREEN;
pub const MOD_TRANS_COLOR: Color32 = Color32::DARK_RED;
