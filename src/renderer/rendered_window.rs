use std::{collections::VecDeque, sync::Arc};

// use image::{GenericImageView, ImageBuffer, Pixel};
use skia_safe::{
    canvas::{SaveLayerRec, SrcRectConstraint},
    gpu::SurfaceOrigin,
    image_filters::blur,
    BlendMode, Budgeted, Canvas, Color, EncodedImageFormat, Image, ImageInfo, Paint, Point, Rect,
    SamplingOptions, Surface, SurfaceProps, SurfacePropsFlags,
};
use std::{fs, io::Write, path::Path};

use crate::{
    dimensions::Dimensions,
    editor::Style,
    redraw_scheduler::REDRAW_SCHEDULER,
    renderer::{animation_utils::*, GridRenderer, RendererSettings},
};

#[derive(Clone, Debug)]
pub struct LineFragment {
    pub text: String,
    pub window_left: u64,
    pub window_top: u64,
    pub width: u64,
    pub style: Option<Arc<Style>>,
}

#[derive(Clone, Debug)]
pub enum WindowDrawCommand {
    Position {
        grid_position: (f64, f64),
        grid_size: (u64, u64),
        floating_order: Option<u64>,
    },
    DrawLine(Vec<LineFragment>),
    Scroll {
        top: u64,
        bottom: u64,
        left: u64,
        right: u64,
        rows: i64,
        cols: i64,
    },
    Clear,
    Show,
    Hide,
    Close,
    Viewport {
        top_line: f64,
        bottom_line: f64,
    },
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

pub struct LocatedSnapshot {
    image: Image,
    top_line: u64,
}

#[derive(Copy, Clone)]
struct PositionOverride {
    top_line: u64,
    current_scroll: f32,
}

pub struct RenderedWindow {
    pub surface : Surface,

    pub id: u64,
    pub hidden: bool,
    pub floating_order: Option<u64>,

    pub grid_size: Dimensions,

    grid_start_position: Point,
    pub grid_current_position: Point,
    grid_destination: Point,
    position_t: f32,

    start_scroll: f32,
    pub current_scroll: f32,
    scroll_destination: f32,
    scroll_t: f32,
}

#[derive(Clone, Debug)]
pub struct WindowDrawDetails {
    pub id: u64,
    pub region: Rect,
    pub floating_order: Option<u64>,
}

impl RenderedWindow {
    pub fn new(
        parent_canvas: &mut Canvas,
        grid_renderer: &GridRenderer,
        id: u64,
        grid_position: Point,
        grid_size: Dimensions,
    ) -> RenderedWindow {
        let surface = build_window_surface_with_grid_size(parent_canvas, grid_renderer, grid_size);

        RenderedWindow {
            surface: surface,
            id,
            hidden: false,
            floating_order: None,

            grid_size,

            grid_start_position: grid_position,
            grid_current_position: grid_position,
            grid_destination: grid_position,
            position_t: 2.0, // 2.0 is out of the 0.0 to 1.0 range and stops animation

            start_scroll: 0.0,
            current_scroll: 0.0,
            scroll_destination: 0.0,
            scroll_t: 2.0, // 2.0 is out of the 0.0 to 1.0 range and stops animation
        }
    }

    pub fn pixel_region(&self, font_dimensions: Dimensions) -> Rect {
        let current_pixel_position = Point::new(
            self.grid_current_position.x * font_dimensions.width as f32,
            self.grid_current_position.y * font_dimensions.height as f32,
        );

        let image_size: (i32, i32) = (self.grid_size * font_dimensions).into();

        Rect::from_point_and_size(current_pixel_position, image_size)
    }

    pub fn draw(
        &mut self,
        root_canvas: &mut Canvas,
        settings: &RendererSettings,
        default_background: Color,
        font_dimensions: Dimensions,
        dt: f32,
    ) -> WindowDrawDetails {
        let pixel_region = self.pixel_region(font_dimensions);
        //
        // root_canvas.save();
        // root_canvas.clip_rect(&pixel_region, None, Some(false));
        // root_canvas.clear(default_background);
        //
        // let mut paint = Paint::default();
        // // We want each surface to overwrite the one underneath and will use layers to ensure
        // // only lower priority surfaces will get clobbered and not the underlying windows
        // paint.set_blend_mode(BlendMode::Src);
        // paint.set_anti_alias(false);
        //
        // // Save layer so that setting the blend mode doesn't effect the blur
        // root_canvas.save_layer(&SaveLayerRec::default());
        //
        // let a = 255;
        // paint.set_color(default_background.with_a(a));
        // root_canvas.draw_rect(pixel_region, &paint);
        //
        // paint.set_color(Color::from_argb(255, 255, 255, 255));

        // let font_height = font_dimensions.height;

        //
        //
        // let surface = build_window_surface_with_grid_size(root_canvas,);
        // Draw current surface
        let snapshot = self.surface.image_snapshot();

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


        WindowDrawDetails {
            id: self.id,
            region: pixel_region,
            floating_order: self.floating_order,
        }
    }
}
