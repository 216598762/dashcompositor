//! `dashcompositor` CLI -- terminal-fit layer-stack demo.
//!
//! Demonstrates that a backend (this binary) can:
//! 1. Detect the host terminal's cell-grid size via
//!    [`dashcompositor::TerminalSize`].
//! 2. Build a [`dashcompositor::LayerStack`], add and remove layers,
//!    and control their opacity / visibility / z-order override.
//! 3. Render the stack into a framebuffer auto-sized to the terminal
//!    via [`dashcompositor::LayerStack::render_to_current_terminal`].
//! 4. Report the terminal size back through the API.

use dashcompositor::{LayerStack, SolidColor, TerminalSize};

fn main() {
    let size = TerminalSize::current();
    eprintln!(
        "dashcompositor v0.3.0 -- terminal-fit compositor: \
host terminal = {cols} cols x {rows} rows",
        cols = size.cols,
        rows = size.rows,
    );

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

    // 3. Auto-fit the framebuffer to the host terminal and render.
    let (fb, reported) = stack.render_to_current_terminal();
    assert_eq!(reported.cols as u32, fb.width());
    assert_eq!(reported.rows as u32, fb.height());
    eprintln!(
        "rendered {}x{} framebuffer ({} pixels, {} layer(s))",
        fb.width(),
        fb.height(),
        fb.pixels().len(),
        stack.len(),
    );

    // 4. Control at will: hide foreground, remove background, re-add
    //    a new accent layer with a z-override, re-render.
    if let Some(entry) = stack.get_mut(fg) {
        entry.set_visible(false);
    }
    let _ = stack.remove(bg);
    let accent = stack.push(SolidColor::new(0, 0, 255, 255).with_name("accent-blue"));
    if let Some(entry) = stack.get_mut(accent) {
        entry.set_z_override(100);
    }
    let (fb2, _) = stack.render_to_current_terminal();
    eprintln!(
        "after control: rendered {}x{} framebuffer ({} pixels, {} layer(s))",
        fb2.width(),
        fb2.height(),
        fb2.pixels().len(),
        stack.len(),
    );
}
