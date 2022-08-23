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

fn build_window_surface(parent_canvas: &mut Canvas, pixel_size: (i32, i32)) -> Surface {
    let mut context = parent_canvas.recording_context().unwrap();
    let budgeted = Budgeted::Yes;
    let parent_image_info = parent_canvas.image_info();
    let image_info = ImageInfo::new(
        pixel_size,
        parent_image_info.color_type(),
        parent_image_info.alpha_type(),
        parent_image_info.color_space(),
    );
    let surface_origin = SurfaceOrigin::TopLeft;
    // subpixel layout (should be configurable/obtained from fontconfig)
    let props = SurfaceProps::new(SurfacePropsFlags::default(), skia_safe::PixelGeometry::RGBH);
    Surface::new_render_target(
        &mut context,
        budgeted,
        &image_info,
        None,
        surface_origin,
        Some(&props),
        None,
    )
    .expect("Could not create surface")
}

fn build_window_surface_with_grid_size(
    parent_canvas: &mut Canvas,
    grid_renderer: &GridRenderer,
    grid_size: Dimensions,
) -> Surface {
    let mut surface = build_window_surface(
        parent_canvas,
        (grid_size * grid_renderer.font_dimensions).into(),
    );

    let canvas = surface.canvas();
    canvas.clear(grid_renderer.get_default_background());
    surface
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

    // rendered_window: RenderedWindow,
    // rendered_windows: HashMap<u64, RenderedWindow>,
    pub window_regions: Vec<WindowDrawDetails>,
}

impl Renderer {
    pub fn new(root_canvas: &mut Canvas, scale_factor: f64) -> Self {
        let grid_renderer = GridRenderer::new(scale_factor);
        // let rendered_window = RenderedWindow::new(
        //     root_canvas,
        //     &grid_renderer,
        //     0,
        //     (0.0, 0.0).into(),
        //     (100, 100).into(),
        // );

        let window_regions = Vec::new();

        Renderer {
            // rendered_window,
            grid_renderer,
            window_regions,
        }
    }

    pub fn font_names(&self) -> Vec<String> {
        self.grid_renderer.font_names()
    }

    #[allow(clippy::needless_collect)]
    pub fn draw_frame(
        &mut self,
        root_canvas: &mut Canvas,
        line_fragments: Vec<LineFragment>,
    ) -> bool {
        let mut surface = build_window_surface_with_grid_size(
            root_canvas,
            &self.grid_renderer,
            (100, 100).into(),
        );

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
            self.grid_renderer
                .draw_background(canvas, grid_position, *width, style, false);
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
            self.grid_renderer
                .draw_foreground(canvas, text, grid_position, width, &style);
        }
        canvas.restore();

        let default_background = self.grid_renderer.get_default_background();
        let font_dimensions = self.grid_renderer.font_dimensions;

        let transparency = { SETTINGS.get::<WindowSettings>().transparency };
        root_canvas.clear(default_background.with_a((255.0 * transparency) as u8));
        root_canvas.save();
        root_canvas.reset_matrix();

        // let clip_rect = self.rendered_window.pixel_region(font_dimensions);
        // root_canvas.clip_rect(&clip_rect, None, Some(false));

        // let settings = SETTINGS.get::<RendererSettings>();

        // let grid_size: Dimensions = (100, 100).into();
        // let current_pixel_position = Point::new(0.0, 0.0);
        //
        // let image_size: (i32, i32) = (grid_size * font_dimensions).into();

        // let pixel_region = Rect::from_point_and_size(current_pixel_position, image_size);





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

        root_canvas.restore();
        false
    }
}
