// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::{Extent2D, Texture};

pub struct FramebufferBuilder<'a> {
    extent: Extent2D,
    color_texture: Option<&'a Texture>,
    depth_texture: Option<&'a Texture>,
}

impl<'a> FramebufferBuilder<'a> {
    pub fn new() -> Self {
        FramebufferBuilder {
            extent: Extent2D::default(),
            color_texture: None,
            depth_texture: None,
        }
    }

    pub fn extent(mut self, extent: Extent2D) -> Self {
        self.extent = extent;
        self
    }

    pub fn color_attachment(mut self, color_texture: &'a Texture) -> Self {
        self.color_texture = Some(color_texture);
        self
    }

    pub fn depth_attachment(mut self, depth_texture: &'a Texture) -> Self {
        self.depth_texture = Some(depth_texture);
        self
    }

    pub fn build(self) -> Framebuffer {
        let mut handle = 0;
        unsafe { gl::GenFramebuffers(1, &mut handle as _) };

        let mut framebuffer = Framebuffer::new(handle, self.extent);
        framebuffer.bind();

        framebuffer.set_color_attachment(&self.color_texture);
        framebuffer.set_depth_attachment(&self.depth_texture);

        if !framebuffer.is_complete() {
            println!("Framebuffer is not complete");
            super::gl_check();
        }

        framebuffer
    }
}

pub struct Framebuffer {
    /// This is 0 for the default framebuffer
    handle: u32,

    /// Physical extent of the framebuffer
    pub extent: Extent2D,

    /// Used in certain fragment shaders
    pub virtual_extent: Extent2D,
}

impl Framebuffer {
    pub fn builder<'a>() -> FramebufferBuilder<'a> {
        FramebufferBuilder::new()
    }

    pub fn default(extent: Extent2D) -> Self {
        Self {
            handle: 0,
            extent,
            virtual_extent: extent,
        }
    }

    fn new(handle: u32, extent: Extent2D) -> Self {
        Self {
            handle,
            extent,
            virtual_extent: extent,
        }
    }

    fn set_attachment(&mut self, attachment_type: gl::types::GLenum, texture: &Option<&Texture>) {
        let handle = match texture {
            Some(texture) => texture.handle,
            None => gl::NONE,
        };

        unsafe {
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment_type, gl::TEXTURE_2D, handle, 0)
        };
    }

    fn set_color_attachment(&mut self, color_texture: &Option<&Texture>) {
        self.set_attachment(gl::COLOR_ATTACHMENT0, color_texture);
    }

    // We need to use a depth texture to sample from
    fn set_depth_attachment(&mut self, depth_texture: &Option<&Texture>) {
        self.set_attachment(gl::DEPTH_ATTACHMENT, depth_texture);
    }

    fn is_complete(&self) -> bool {
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        status == gl::FRAMEBUFFER_COMPLETE
    }

    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.handle) };
    }

    pub fn bind_default() {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };
    }

    pub fn unbind(&self) {
        Self::bind_default();
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.handle as _) };
    }
}
