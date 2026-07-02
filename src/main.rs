//! `dashcompositor` CLI — first-subsystem demo.
//!
//! Demonstrates that a backend (this binary) can build a
//! [`dashcompositor::LayerStack`], add and remove layers, control
//! their opacity / visibility / z-order override, and render the
//! result into an RGBA [`dashcompositor::FrameBuffer`].
//!
//! No protocol encoder is implemented yet (`AGENTS.md` §3 keeps the
//! `kittage` / `icy_sixel` candidate crates un-adopted), so the
//! composited framebuffer is not emitted to stdout; instead it is
//! summarised as channel sums so a human can verify the compositing
//! path works end-to-end.

use dashcompositor::{FrameBuffer, LayerStack, SolidColor};

fn main() {
    let mut stack = LayerStack::new();

    // 1. Add a full-frame red background at z=0.
    let bg = stack.push(SolidColor::new(255, 0, 0, 255).with_name("background-red"));

    // 2. Add a translucent green overlay at z=10.
    let fg = stack.push(
        SolidColor::new(0, 255, 0, 128)
            .with_z(10)
            .with_name("overlay-green"),
    );
    if let Some(entry) = stack.get_mut(fg) {
        entry.set_opacity(0.5);
    }

    // 3. Render the initial state.
    let mut fb = FrameBuffer::new(4, 2);
    stack.render(&mut fb);
    let (r, g, b, a) = channel_sums(fb.pixels());
    eprintln!(
        "dashcompositor demo v0.2.0 — first subsystem (layer stack): \
stack len={}, framebuffer {}x{}, channel sums R={r} G={g} B={b} A={a}",
        stack.len(),
        fb.width(),
        fb.height(),
    );

    // 4. Control at will: hide foreground, remove background, re-add
    //    a new accent layer with a z-override, render again.
    if let Some(entry) = stack.get_mut(fg) {
        entry.set_visible(false);
    }
    let _ = stack.remove(bg);
    let accent = stack.push(SolidColor::new(0, 0, 255, 255).with_name("accent-blue"));
    if let Some(entry) = stack.get_mut(accent) {
        entry.set_z_override(100);
    }

    let mut fb2 = FrameBuffer::new(4, 2);
    stack.render(&mut fb2);
    let (r2, g2, b2, a2) = channel_sums(fb2.pixels());
    eprintln!(
        "after control: stack len={}, channel sums R={r2} G={g2} B={b2} A={a2}",
        stack.len(),
    );
}

/// Returns the per-channel sum of an RGBA pixel slice.
fn channel_sums(pixels: &[[u8; 4]]) -> (u32, u32, u32, u32) {
    pixels
        .iter()
        .fold((0u32, 0u32, 0u32, 0u32), |(r, g, b, a), px| {
            (
                r + u32::from(px[0]),
                g + u32::from(px[1]),
                b + u32::from(px[2]),
                a + u32::from(px[3]),
            )
        })
}
