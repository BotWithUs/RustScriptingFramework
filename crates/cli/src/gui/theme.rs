#![allow(dead_code)]

use imgui::{Context, StyleColor};

// Background & surface (dark Bootstrap-style)
pub const BG: [f32; 4] = [0x12 as f32 / 255.0, 0x12 as f32 / 255.0, 0x17 as f32 / 255.0, 1.0];
pub const INPUT_BG: [f32; 4] = [0x1c as f32 / 255.0, 0x1e as f32 / 255.0, 0x26 as f32 / 255.0, 1.0];

// Text
pub const TEXT: [f32; 4] = [0xf0 as f32 / 255.0, 0xf0 as f32 / 255.0, 0xf0 as f32 / 255.0, 1.0];
pub const DIM_TEXT: [f32; 4] = [0x80 as f32 / 255.0, 0x80 as f32 / 255.0, 0x88 as f32 / 255.0, 1.0];

// Accent - BotWithUs brand blue (#4a90e2)
pub const ACCENT: [f32; 4] = [0x4a as f32 / 255.0, 0x90 as f32 / 255.0, 0xe2 as f32 / 255.0, 1.0];

// Status colors
pub const GREEN: [f32; 4] = [0x28 as f32 / 255.0, 0xa7 as f32 / 255.0, 0x45 as f32 / 255.0, 1.0];
pub const RED: [f32; 4] = [0xdc as f32 / 255.0, 0x35 as f32 / 255.0, 0x45 as f32 / 255.0, 1.0];
pub const YELLOW: [f32; 4] = [0xf0 as f32 / 255.0, 0xad as f32 / 255.0, 0x4e as f32 / 255.0, 1.0];
pub const CYAN: [f32; 4] = [0x67 as f32 / 255.0, 0xe8 as f32 / 255.0, 0xf9 as f32 / 255.0, 1.0];
pub const MAGENTA: [f32; 4] = [0xc0 as f32 / 255.0, 0x84 as f32 / 255.0, 0xfc as f32 / 255.0, 1.0];

fn accent_alpha(a: f32) -> [f32; 4] {
    [ACCENT[0], ACCENT[1], ACCENT[2], a]
}

/// Apply the BotWithUs dark theme to the ImGui context.
pub fn apply(imgui: &mut Context, scale: f32) {
    let style = imgui.style_mut();

    // Rounding
    style.window_rounding = 0.0;
    style.frame_rounding = 4.0 * scale;
    style.scrollbar_rounding = 4.0 * scale;
    style.grab_rounding = 2.0 * scale;
    style.tab_rounding = 4.0 * scale;

    // Padding
    style.window_padding = [8.0 * scale, 8.0 * scale];
    style.frame_padding = [6.0 * scale, 4.0 * scale];
    style.item_spacing = [8.0 * scale, 4.0 * scale];
    style.item_inner_spacing = [4.0 * scale, 4.0 * scale];
    style.scrollbar_size = 12.0 * scale;

    // Window & child backgrounds
    style[StyleColor::WindowBg] = BG;
    style[StyleColor::ChildBg] = BG;
    style[StyleColor::PopupBg] = [INPUT_BG[0], INPUT_BG[1], INPUT_BG[2], 0.95];

    // Borders
    style[StyleColor::Border] = [1.0, 1.0, 1.0, 0.1];

    // Input frames
    style[StyleColor::FrameBg] = INPUT_BG;
    style[StyleColor::FrameBgHovered] = accent_alpha(0.15);
    style[StyleColor::FrameBgActive] = accent_alpha(0.25);

    // Title bar
    style[StyleColor::TitleBg] = INPUT_BG;
    style[StyleColor::TitleBgActive] = [0x1a as f32 / 255.0, 0x1e as f32 / 255.0, 0x2c as f32 / 255.0, 1.0];

    // Text
    style[StyleColor::Text] = TEXT;
    style[StyleColor::TextDisabled] = DIM_TEXT;

    // Buttons (brand blue)
    style[StyleColor::Button] = accent_alpha(0.65);
    style[StyleColor::ButtonHovered] = accent_alpha(0.85);
    style[StyleColor::ButtonActive] = accent_alpha(1.0);

    // Headers / selectable rows
    style[StyleColor::Header] = accent_alpha(0.2);
    style[StyleColor::HeaderHovered] = accent_alpha(0.35);
    style[StyleColor::HeaderActive] = accent_alpha(0.5);

    // Tabs
    style[StyleColor::Tab] = INPUT_BG;
    style[StyleColor::TabHovered] = accent_alpha(0.5);
    style[StyleColor::TabActive] = accent_alpha(0.7);
    style[StyleColor::TabUnfocused] = INPUT_BG;
    style[StyleColor::TabUnfocusedActive] = accent_alpha(0.4);

    // Table
    style[StyleColor::TableHeaderBg] = [0x1a as f32 / 255.0, 0x1e as f32 / 255.0, 0x2c as f32 / 255.0, 1.0];
    style[StyleColor::TableBorderStrong] = [1.0, 1.0, 1.0, 0.1];
    style[StyleColor::TableBorderLight] = [1.0, 1.0, 1.0, 0.06];
    style[StyleColor::TableRowBg] = [0.0, 0.0, 0.0, 0.0];
    style[StyleColor::TableRowBgAlt] = [1.0, 1.0, 1.0, 0.03];

    // Separators
    style[StyleColor::Separator] = [1.0, 1.0, 1.0, 0.1];

    // Scrollbar
    style[StyleColor::ScrollbarBg] = [BG[0], BG[1], BG[2], 0.5];
    style[StyleColor::ScrollbarGrab] = [1.0, 1.0, 1.0, 0.3];
    style[StyleColor::ScrollbarGrabHovered] = [1.0, 1.0, 1.0, 0.5];
    style[StyleColor::ScrollbarGrabActive] = accent_alpha(1.0);

    // Widgets
    style[StyleColor::CheckMark] = accent_alpha(1.0);
    style[StyleColor::SliderGrab] = accent_alpha(0.8);
    style[StyleColor::SliderGrabActive] = accent_alpha(1.0);
    style[StyleColor::PlotHistogram] = accent_alpha(1.0);
    style[StyleColor::PlotHistogramHovered] = [ACCENT[0] + 0.1, ACCENT[1] + 0.1, ACCENT[2] + 0.1, 1.0];
    style[StyleColor::TextSelectedBg] = accent_alpha(0.3);
}
