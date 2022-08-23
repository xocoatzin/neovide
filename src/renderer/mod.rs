pub mod animation_utils;
pub mod fonts;
pub mod grid_renderer;
mod rendered_window;

use skia_safe::{
    gpu::SurfaceOrigin,
    Budgeted, Canvas,  EncodedImageFormat,  ImageInfo, 
     Surface, SurfaceProps, SurfacePropsFlags,
};
use std::{fs, io::Write, path::Path};

use crate::{
    dimensions::Dimensions,
    bridge::EditorMode,
    editor::{Cursor, Style},
    settings::*,
    WindowSettings,
};

pub use fonts::caching_shaper::CachingShaper;
pub use grid_renderer::GridRenderer;
pub use rendered_window::{LineFragment, RenderedWindow, WindowDrawCommand, WindowDrawDetails};

#[derive(SettingGroup, Clone)]
pub struct RendererSettings {
    position_animation_length: f32,
    scroll_animation_length: f32,
    floating_opacity: f32,
    floating_blur: bool,
    floating_blur_amount_x: f32,
    floating_blur_amount_y: f32,
    debug_renderer: bool,
    underline_automatic_scaling: bool,
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self {
            position_animation_length: 0.15,
            scroll_animation_length: 0.3,
            floating_opacity: 0.7,
            floating_blur: true,
            floating_blur_amount_x: 2.0,
            floating_blur_amount_y: 2.0,
            debug_renderer: false,
            underline_automatic_scaling: false,
        }
    }
}


#[derive(Clone, Debug)]
pub enum DrawCommand {
    CloseWindow(u64),
    Window {
        grid_id: u64,
        command: WindowDrawCommand,
    },
    UpdateCursor(Cursor),
    FontChanged(String),
    DefaultStyleChanged(Style),
    ModeChanged(EditorMode),
}

pub struct Renderer {
    pub grid_renderer: GridRenderer,
}

impl Renderer {
    pub fn new(root_canvas: &mut Canvas, scale_factor: f64) -> Self {
        let grid_renderer = GridRenderer::new(scale_factor);

        Renderer {
            grid_renderer,
        }
    }

    pub fn font_names(&self) -> Vec<String> {
        self.grid_renderer.font_names()
    }
}
