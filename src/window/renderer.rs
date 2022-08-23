use std::convert::TryInto;

use gl::types::*;
use glutin::PixelFormat;
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, DirectContext, SurfaceOrigin},
    Canvas, ColorType, Surface,
};

type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

fn create_surface(
    windowed_context: &WindowedContext,
    gr_context: &mut DirectContext,
    fb_info: FramebufferInfo,
) -> Surface {
    let pixel_format = windowed_context.get_pixel_format();
    // let pixel_format = PixelFormat {
    //     hardware_accelerated: false,
    //     color_bits: 24,
    //     alpha_bits: 8,
    //     depth_bits: 8,
    //     stencil_bits: 8,
    //     stereoscopy: false,
    //     double_buffer: false,
    //     multisampling: None,
    //     srgb: false,
    // };
    // let size = windowed_context.window().inner_size();
    let size = (800, 600);
    let backend_render_target = BackendRenderTarget::new_gl(
        size,
        pixel_format
            .multisampling
            .map(|s| s.try_into().expect("Could not convert multisampling")),
        pixel_format
            .stencil_bits
            .try_into()
            .expect("Could not convert stencil"),
        fb_info,
    );
    // windowed_context.resize(size.into());
    Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("Could not create skia surface")
}

pub struct SkiaRenderer {
    pub gr_context: DirectContext,
    fb_info: FramebufferInfo,
    surface: Surface,
}

impl SkiaRenderer {
    pub fn new(windowed_context: &WindowedContext) -> SkiaRenderer {
        gl::load_with(|s| windowed_context.get_proc_address(s));

        let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            windowed_context.get_proc_address(name)
        })
        .expect("Could not create interface");

        let mut gr_context = skia_safe::gpu::DirectContext::new_gl(Some(interface), None)
            .expect("Could not create direct context");
        let fb_info = {
            let mut fboid: GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid.try_into().expect("Could not create frame buffer id"),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            }
        };
        let surface = create_surface(windowed_context, &mut gr_context, fb_info);

        SkiaRenderer {
            gr_context,
            fb_info,
            surface,
        }
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.surface.canvas()
    }

    // pub fn resize(&mut self, windowed_context: &WindowedContext) {
    //     self.surface = create_surface(windowed_context, &mut self.gr_context, self.fb_info);
    // }
}
