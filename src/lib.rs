//! `dashcompositor` — layer-based graphics compositor for the terminal.
//!
//! See [`AGENTS.md`](../AGENTS.md) and the [README](../README.md) for
//! project rules and the target architecture.

pub mod compositor;
pub mod encoder;
pub mod framebuffer;
pub mod layer;
pub mod terminal;

pub use compositor::{Compositor, CpuCompositor, LayerStack};
pub use encoder::Protocol;
pub use framebuffer::{blend_over, FrameBuffer};
pub use layer::{Layer, LayerEntry, LayerId, SolidColor};
pub use terminal::TerminalSize;

#[cfg(test)]
mod tests {
    use super::{FrameBuffer, LayerStack, SolidColor};

    #[test]
    fn empty_framebuffer_is_zero_sized_pixels() {
        let fb = FrameBuffer::new(2, 3);
        assert_eq!(fb.width(), 2);
        assert_eq!(fb.height(), 3);
        assert_eq!(fb.pixels().len(), 6);
    }

    #[test]
    fn end_to_end_add_remove_control_render() {
        let mut stack = LayerStack::new();
        let bg = stack.push(SolidColor::new(0, 0, 0, 255).with_name("bg"));
        let fg = stack.push(SolidColor::new(255, 255, 255, 255).with_z(10));

        // Control: fade and hide.
        stack.get_mut(fg).unwrap().set_opacity(0.25);
        stack.get_mut(bg).unwrap().set_visible(false);

        // Remove and re-add.
        assert!(stack.remove(bg).is_some());
        let accent = stack.push(SolidColor::new(255, 0, 0, 255));
        stack.get_mut(accent).unwrap().set_z_override(99);

        // Render.
        let mut fb = FrameBuffer::new(2, 1);
        stack.render(&mut fb);
        // Top of stack is now the accent (red, full alpha).
        assert_eq!(fb.pixels()[0][0], 255);
    }
}
