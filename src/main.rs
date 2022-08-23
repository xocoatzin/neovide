#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Test naming occasionally uses camelCase with underscores to separate sections of
// the test name.
#![cfg_attr(test, allow(non_snake_case))]
#[macro_use]
extern crate neovide_derive;

#[macro_use]
extern crate clap;

mod bridge;
mod channel_utils;
mod cmd_line;
mod dimensions;
mod editor;
mod error_handling;
mod event_aggregator;
mod frame;
mod redraw_scheduler;
mod renderer;
mod running_tracker;
mod settings;
mod window;

// #[cfg(target_os = "windows")]
// mod windows_utils;

#[macro_use]
extern crate derive_new;
// #[macro_use]
// extern crate lazy_static;

// use std::env::args;

use skia_safe::{Color4f, EncodedImageFormat, Surface};
use std::sync::Arc;
use std::{fs, io::Write, path::Path};
// use cmd_line::CmdLineSettings;
// use editor::start_editor;
// use settings::SETTINGS;
use crate::{
    // editor::{Colors, Style, UnderlineStyle},
    renderer::{grid_renderer::GridRenderer, LineFragment},
    // bridge::{ParallelCommand, UiCommand},
    // cmd_line::CmdLineSettings,
    // dimensions::Dimensions,
    // settings::{
    //     load_last_window_settings, save_window_geometry, PersistentWindowSettings, SETTINGS,
    // },
};

// use renderer::{grid_renderer::GridRenderer, Renderer, RendererSettings};
// use window::{KeyboardSettings, WindowSettings};

// pub use channel_utils::*;
// pub use event_aggregator::*;
// pub use running_tracker::*;

#[derive(new, PartialEq, Debug, Clone)]
pub struct Colors {
    pub foreground: Option<Color4f>,
    pub background: Option<Color4f>,
    pub special: Option<Color4f>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum UnderlineStyle {
    Underline,
    UnderDouble,
    UnderDash,
    UnderDot,
    UnderCurl,
}

#[derive(new, Debug, Clone, PartialEq)]
pub struct Style {
    pub colors: Colors,
    #[new(default)]
    pub reverse: bool,
    #[new(default)]
    pub italic: bool,
    #[new(default)]
    pub bold: bool,
    #[new(default)]
    pub strikethrough: bool,
    #[new(default)]
    pub blend: u8,
    #[new(default)]
    pub underline: Option<UnderlineStyle>,
}

impl Style {
    pub fn foreground(&self, default_colors: &Colors) -> Color4f {
        if self.reverse {
            self.colors
                .background
                .unwrap_or_else(|| default_colors.background.unwrap())
        } else {
            self.colors
                .foreground
                .unwrap_or_else(|| default_colors.foreground.unwrap())
        }
    }

    pub fn background(&self, default_colors: &Colors) -> Color4f {
        if self.reverse {
            self.colors
                .foreground
                .unwrap_or_else(|| default_colors.foreground.unwrap())
        } else {
            self.colors
                .background
                .unwrap_or_else(|| default_colors.background.unwrap())
        }
    }

    pub fn special(&self, default_colors: &Colors) -> Color4f {
        self.colors
            .special
            .unwrap_or_else(|| self.foreground(default_colors))
    }
}


fn main() {
    //Will exit if -h or -v
    // if let Err(err) = cmd_line::handle_command_line_arguments(args().collect()) {
    //     eprintln!("{}", err);
    //     return;
    // }

    // WindowSettings::register();
    // RendererSettings::register();
    // KeyboardSettings::register();

    let mut grid_renderer = GridRenderer::new(1.0);
    let line_fragments: Vec<LineFragment> = [
        LineFragment {
            text: "Hello -> =>".to_string(),
            style: None,
            window_left: 0,
            window_top: 0,
            width: 20,
        },
        LineFragment {
            text: "Bye".to_string(),
            style: Some(Arc::new(Style {
                reverse: false,
                italic: true,
                bold: true,
                strikethrough: false,
                blend: 120,
                underline: Some(UnderlineStyle::UnderCurl),
                colors: Colors {
                    foreground: Some(Color4f {
                        a: 1.0,
                        b: 0.2,
                        g: 1.0,
                        r: 1.0,
                    }),
                    background: Some(Color4f {
                        a: 1.0,
                        b: 0.4,
                        g: 0.2,
                        r: 0.2,
                    }),
                    special: Some(Color4f {
                        a: 1.0,
                        b: 0.5,
                        g: 0.5,
                        r: 0.5,
                    }),
                },
            })),
            window_left: 0,
            window_top: 20,
            width: 50,
        },
    ]
    .to_vec();

    // let mut root_canvas = skia_renderer.canvas();
    //
    // let mut surface =
    //     build_window_surface_with_grid_size(root_canvas, &grid_renderer, (100, 100).into());

    let mut surface = Surface::new_raster_n32_premul((2000, 1000)).unwrap();

    let canvas = surface.canvas();

    canvas.save();
    for line_fragment in line_fragments.iter() {
        let LineFragment {
            window_left,
            window_top,
            width,
            style,
            ..
        } = line_fragment;
        let grid_position = (*window_left, *window_top);
        grid_renderer.draw_background(canvas, grid_position, *width, style, false);
    }

    for line_fragment in line_fragments.into_iter() {
        let LineFragment {
            text,
            window_left,
            window_top,
            width,
            style,
        } = line_fragment;
        let grid_position = (window_left, window_top);
        grid_renderer.draw_foreground(canvas, text, grid_position, width, &style);
    }
    canvas.restore();

    let snapshot = surface.image_snapshot();

    let data = snapshot.encode_to_data(EncodedImageFormat::PNG).unwrap();
    let ext = "png";
    let path = Path::new("/tmp");
    let name = "example";

    fs::create_dir_all(&path).expect("failed to create directory");

    let mut file_path = path.join(name);
    file_path.set_extension(ext);

    let mut file = fs::File::create(file_path).expect("failed to create file");
    file.write_all(data.as_bytes())
        .expect("failed to write to file");
}
