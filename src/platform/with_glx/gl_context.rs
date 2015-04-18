use glx;
use xlib::*;
use libc::*;
use glx::types::{GLXContext, GLXDrawable, GLXFBConfig, GLXPixmap};
use geom::{Size2D};
use platform::with_glx::utils::{create_offscreen_pixmap_backed_context};

use GLContextMethods;
use GLContextCapabilities;
use GLContextAttributes;
use DrawBuffer;

pub struct GLContext {
    native_context: GLXContext,
    native_display: *mut glx::types::Display,
    native_drawable: GLXDrawable,
    // DrawBuffer must be created with the current context up and running
    pub draw_buffer: Option<DrawBuffer>,
    pub attributes: GLContextAttributes,
    pub capabilities: GLContextCapabilities
}

impl GLContext {
    pub fn new(share_context: Option<&GLContext>,
               display: *mut glx::types::Display,
               drawable: GLXDrawable,
               framebuffer_config: GLXFBConfig,
               attributes: Option<GLContextAttributes>)
        -> Result<GLContext, &'static str> {

        let shared = match share_context {
            Some(ctx) => ctx.as_native_glx_context(),
            None      => 0 as GLXContext
        };

        let native = unsafe { glx::CreateNewContext(display, framebuffer_config, glx::RGBA_TYPE as c_int, shared, 1 as glx::types::Bool) };

        // FIXME: This should be:
        // if native == (0 as *const c_void) {
        // but that way compilation fails with error:
        //  expected `*const libc::types::common::c95::c_void`,
        //     found `*const libc::types::common::c95::c_void`
        // (expected enum `libc::types::common::c95::c_void`,
        //     found a different enum `libc::types::common::c95::c_void`) [E0308]
        if (native as *const c_void) == (0 as *const c_void) {
            unsafe { glx::DestroyPixmap(display, drawable as GLXPixmap) };
            return Err("Error creating native glx context");
        }

        // FIXME(ecoal95): This feels kind of weird, but we must ensure
        //   the context is current in order to detect capabilities.
        //   Another option could be to use `Option<GLContextCapabilities>`
        //   initialized to none and using some kind of
        //   context.init/context.detect_capabilities, and the extra option
        //   will force to `unwrap` each time.
        unsafe {
            glx::MakeCurrent(display, drawable, native);
        }

        Ok(GLContext {
            native_context: native,
            native_display: display,
            native_drawable: drawable,
            draw_buffer: None,
            attributes: attributes.unwrap_or(GLContextAttributes::any()),
            capabilities: GLContextCapabilities::detect()
        })
    }

    fn as_native_glx_context(&self) -> GLXContext {
        self.native_context
    }
}

impl Drop for GLContext {
    fn drop(&mut self) {
        self.make_current().unwrap();
        unsafe {
            glx::DestroyContext(self.native_display, self.native_context);
            glx::DestroyPixmap(self.native_display, self.native_drawable as GLXPixmap);
        }
    }
}

impl GLContextMethods for GLContext {
    // FIXME(ecoal95): In Gecko this is created with a dummy size
    //   and later resized with corresponding checks about max fb length
    //   we're not handling resizing yet though
    fn create_headless(size: Size2D<i32>) -> Result<GLContext, &'static str> {
        create_offscreen_pixmap_backed_context(size)
    }

    fn make_current(&self) -> Result<(), &'static str> {
        unsafe {
            if glx::GetCurrentContext() != self.native_context
                && glx::MakeCurrent(self.native_display,
                                    self.native_drawable,
                                    self.native_context) == 0 {
                Err("glx::MakeContextCurrent")
            } else {
                Ok(())
            }
        }
    }
}

