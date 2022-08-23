// mod keyboard_manager;
// mod mouse_manager;
mod renderer;
mod settings;
use std::{collections::HashMap, sync::Arc};

#[cfg(target_os = "macos")]
mod draw_background;

use skia_safe::Color4f;
use std::time::{Duration, Instant};

use glutin::{
    self,
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{self, Fullscreen, Icon},
    ContextBuilder, GlProfile, WindowedContext,
};
use log::trace;
use tokio::sync::mpsc::UnboundedReceiver;

#[cfg(target_os = "macos")]
use glutin::platform::macos::WindowBuilderExtMacOS;

#[cfg(target_os = "macos")]
use draw_background::draw_background;

#[cfg(target_os = "linux")]
use glutin::platform::unix::WindowBuilderExtUnix;

use image::{load_from_memory, GenericImageView, Pixel};
// use keyboard_manager::KeyboardManager;
// use mouse_manager::MouseManager;
use renderer::SkiaRenderer;

use crate::{
    bridge::{ParallelCommand, UiCommand},
    cmd_line::CmdLineSettings,
    dimensions::Dimensions,
    editor::{EditorCommand, Style, Colors, UnderlineStyle},
    event_aggregator::EVENT_AGGREGATOR,
    frame::Frame,
    redraw_scheduler::REDRAW_SCHEDULER,
    renderer::{LineFragment, Renderer},
    running_tracker::*,
    settings::{
        load_last_window_settings, save_window_geometry, PersistentWindowSettings, SETTINGS,
    },
};
pub use settings::{KeyboardSettings, WindowSettings};

const MIN_WINDOW_WIDTH: u64 = 20;
const MIN_WINDOW_HEIGHT: u64 = 6;

pub struct GlutinWindowWrapper {
    // windowed_context: WindowedContext<glutin::PossiblyCurrent>,
    skia_renderer: SkiaRenderer,
    renderer: Renderer,
    // keyboard_manager: KeyboardManager,
    // mouse_manager: MouseManager,
    // title: String,
    // fullscreen: bool,
    // saved_inner_size: PhysicalSize<u32>,
    // saved_grid_size: Option<Dimensions>,
    // size_at_startup: PhysicalSize<u32>,
    // window_command_receiver: UnboundedReceiver<WindowCommand>,
}

impl GlutinWindowWrapper {
    // pub fn send_font_names(&self) {
    //     let font_names = self.renderer.font_names();
    //     EVENT_AGGREGATOR.send(UiCommand::Parallel(ParallelCommand::DisplayAvailableFonts(
    //         font_names,
    //     )));
    // }

    // pub fn handle_quit(&mut self) {
    //     RUNNING_TRACKER.quit("window closed");
    // }

    // pub fn handle_event(&mut self, event: Event<()>) {
    //     self.renderer.handle_event(&event);
    // }

    pub fn draw_frame(&mut self, dt: f32) {
        // let window = self.windowed_context.window();
        let mut font_changed = false;

        // if REDRAW_SCHEDULER.should_draw() || SETTINGS.get::<WindowSettings>().no_idle {
        //     font_changed = self.renderer.draw_frame(self.skia_renderer.canvas());
        //     self.skia_renderer.gr_context.flush(None);
        //     // self.windowed_context.swap_buffers().unwrap();
        // }
        //

        // Wait until fonts are loaded, so we can set proper window size.
        // if !self.renderer.grid_renderer.is_ready {
        //     return;
        // }

        // let new_size = window.inner_size();

        let settings = SETTINGS.get::<CmdLineSettings>();

        // if self.saved_grid_size.is_none() && !false {
        //     window.set_inner_size(
        //         self.renderer
        //             .grid_renderer
        //             .convert_grid_to_physical(settings.geometry),
        //     );
        //     self.saved_grid_size = Some(settings.geometry);
        //     // Font change at startup is ignored, so grid size (and startup screen) could be preserved.
        //     // But only when not resized yet. With maximized or resized window we should redraw grid.
        //     font_changed = false;
        // }
        //
        // if self.saved_inner_size != new_size || font_changed {
        //     self.saved_inner_size = new_size;
        //     self.handle_new_grid_size(new_size);
        //     self.skia_renderer.resize(&self.windowed_context);
        // }
    }

    // fn handle_new_grid_size(&mut self, new_size: PhysicalSize<u32>) {
    //     let grid_size = self
    //         .renderer
    //         .grid_renderer
    //         .convert_physical_to_grid(new_size);
    //
    //     EVENT_AGGREGATOR.send(UiCommand::Parallel(ParallelCommand::Resize {
    //         width: grid_size.width,
    //         height: grid_size.height,
    //     }));
    // }

    // fn handle_scale_factor_update(&mut self, scale_factor: f64) {
    //     self.renderer
    //         .grid_renderer
    //         .handle_scale_factor_update(scale_factor);
    //     EVENT_AGGREGATOR.send(EditorCommand::RedrawScreen);
    // }
}

pub fn create_window() {
    let event_loop = EventLoop::new();

    // let cmd_line_settings = SETTINGS.get::<CmdLineSettings>();

    let winit_window_builder = window::WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false);

    let windowed_context = ContextBuilder::new()
        .with_pixel_format(24, 8)
        .with_stencil_buffer(8)
        .with_gl_profile(GlProfile::Core)
        .with_vsync(false)
        // .with_srgb(cmd_line_settings.srgb)
        .build_windowed(winit_window_builder, &event_loop)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let window = windowed_context.window();
    // let initial_size = window.inner_size();

    let scale_factor = windowed_context.window().scale_factor();
    let mut skia_renderer = SkiaRenderer::new(&windowed_context);
    let mut renderer = Renderer::new(skia_renderer.canvas(), scale_factor);
    // let saved_inner_size = window.inner_size();

    let fragments: Vec<LineFragment> = [
        LineFragment {
            text: "Hello".to_string(),
            style: None,
            window_left: 0,
            window_top: 0,
            width: 20,
        },
        LineFragment {
            text: "Bye".to_string(),
            style: Some(
                Arc::new(
                    Style {
                        reverse: false,
                        italic: true,
                        bold: true,
                        strikethrough: false,
                        blend: 120,
                        underline: Some(UnderlineStyle::UnderCurl),
                        colors: Colors {
                            foreground: Some(
                                Color4f {
                                    a: 1.0,
                                    b: 0.2,
                                    g: 1.0,
                                    r: 1.0,
                                }
                            ),
                            background: Some(
                                Color4f {
                                    a: 1.0,
                                    b: 0.4,
                                    g: 0.2,
                                    r: 0.2,
                                }
                            ),
                            special: Some(
                                Color4f {
                                    a: 1.0,
                                    b: 0.5,
                                    g: 0.5,
                                    r: 0.5,
                                }
                            ),
                        }
                    }
                )
            ),
            window_left: 0,
            window_top: 20,
            width: 50,
        },
    ]
    .to_vec();

    renderer.draw_frame(skia_renderer.canvas(), fragments);
    // skia_renderer.gr_context.flush(None);
}
//
//    Get colors lua
//    return fn.synIDattr(fn.synIDtrans(fn.hlID(group)), attr)
//
// synIDattr({synID}, {what} [, {mode}])			*synIDattr()*
// 		The result is a String, which is the {what} attribute of
// 		syntax ID {synID}.  This can be used to obtain information
// 		about a syntax item.
// 		{mode} can be "gui", "cterm" or "term", to get the attributes
// 		for that mode.  When {mode} is omitted, or an invalid value is
// 		used, the attributes for the currently active highlighting are
// 		used (GUI, cterm or term).
// 		Use synIDtrans() to follow linked highlight groups.
// 		{what}		result
// 		"name"		the name of the syntax item
// 		"fg"		foreground color (GUI: color name used to set
// 				the color, cterm: color number as a string,
// 				term: empty string)
// 		"bg"		background color (as with "fg")
// 		"font"		font name (only available in the GUI)
// 				|highlight-font|
// 		"sp"		special color (as with "fg") |highlight-guisp|
// 		"fg#"		like "fg", but for the GUI and the GUI is
// 				running the name in "#RRGGBB" form
// 		"bg#"		like "fg#" for "bg"
// 		"sp#"		like "fg#" for "sp"
// 		"bold"		"1" if bold
// 		"italic"	"1" if italic
// 		"reverse"	"1" if reverse
// 		"inverse"	"1" if inverse (= reverse)
// 		"standout"	"1" if standout
// 		"underline"	"1" if underlined
// 		"underlineline"	"1" if double underlined
// 		"undercurl"	"1" if undercurled
// 		"underdot"	"1" if dotted underlined
// 		"underdash"	"1" if dashed underlined
// 		"strikethrough" "1" if struckthrough
//
// 		Example (echoes the color of the syntax item under the
// 		cursor): >
// 	:echo synIDattr(synIDtrans(synID(line("."), col("."), 1)), "fg")
// <
// 		Can also be used as a |method|: >
// 	:echo synID(line("."), col("."), 1)->synIDtrans()->synIDattr("fg")
