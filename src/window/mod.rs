// mod keyboard_manager;
// mod mouse_manager;
mod renderer;
mod settings;
use std::{collections::HashMap, sync::Arc};

#[cfg(target_os = "macos")]
mod draw_background;

use skia_safe::Color4f;
use std::time::{Duration, Instant};

use skia_safe::{
    gpu::SurfaceOrigin, Budgeted, Canvas, EncodedImageFormat, ImageInfo, Surface, SurfaceProps,
    SurfacePropsFlags,
};
use std::{fs, io::Write, path::Path};

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
    editor::{Colors, EditorCommand, Style, UnderlineStyle},
    event_aggregator::EVENT_AGGREGATOR,
    frame::Frame,
    redraw_scheduler::REDRAW_SCHEDULER,
    renderer::{grid_renderer::GridRenderer, LineFragment, Renderer},
    running_tracker::*,
    settings::{
        load_last_window_settings, save_window_geometry, PersistentWindowSettings, SETTINGS,
    },
};
pub use settings::{KeyboardSettings, WindowSettings};

// fn build_window_surface(parent_canvas: &mut Canvas, pixel_size: (i32, i32)) -> Surface {
//     let mut context = parent_canvas.recording_context().unwrap();
//     let budgeted = Budgeted::Yes;
//     let parent_image_info = parent_canvas.image_info();
//     let image_info = ImageInfo::new(
//         pixel_size,
//         parent_image_info.color_type(),
//         parent_image_info.alpha_type(),
//         parent_image_info.color_space(),
//     );
//     let surface_origin = SurfaceOrigin::TopLeft;
//     // subpixel layout (should be configurable/obtained from fontconfig)
//     let props = SurfaceProps::new(SurfacePropsFlags::default(), skia_safe::PixelGeometry::RGBH);
//     Surface::new_render_target(
//         &mut context,
//         budgeted,
//         &image_info,
//         None,
//         surface_origin,
//         Some(&props),
//         None,
//     )
//     .expect("Could not create surface")
// }

// fn build_window_surface_with_grid_size(
//     parent_canvas: &mut Canvas,
//     grid_renderer: &GridRenderer,
//     grid_size: Dimensions,
// ) -> Surface {
//     let mut surface = build_window_surface(
//         parent_canvas,
//         (grid_size * grid_renderer.font_dimensions).into(),
//     );
//
//     let canvas = surface.canvas();
//     canvas.clear(grid_renderer.get_default_background());
//     surface
// }

pub fn create_window() {
    // let event_loop = EventLoop::new();

    // let cmd_line_settings = SETTINGS.get::<CmdLineSettings>();

    // let winit_window_builder = window::WindowBuilder::new()
    //     .with_transparent(true)
    //     .with_decorations(false);

    // let windowed_context = ContextBuilder::new()
    //     .with_pixel_format(24, 8)
    //     .with_stencil_buffer(8)
    //     .with_gl_profile(GlProfile::Core)
    //     .with_vsync(false)
    //     // .with_srgb(cmd_line_settings.srgb)
    //     .build_windowed(winit_window_builder, &event_loop)
    //     .unwrap();
    // let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    //
    // let scale_factor = windowed_context.window().scale_factor();
    // let mut skia_renderer = SkiaRenderer::new(&windowed_context);
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

    // let default_background = grid_renderer.get_default_background();
    // let font_dimensions = grid_renderer.font_dimensions;
    //
    // let transparency = { SETTINGS.get::<WindowSettings>().transparency };
    // root_canvas.clear(default_background.with_a((255.0 * transparency) as u8));
    // root_canvas.save();
    // root_canvas.reset_matrix();

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

    // root_canvas.restore();
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
