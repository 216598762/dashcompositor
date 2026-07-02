//! Layer — a single compositable layer in the compositor.
//!
//! Concrete [`Layer`] implementations include raster images, text
//! glyphs, vector shapes, and solid-colour fills. Each layer is
//! identified by a [`LayerId`] when wrapped in a [`LayerEntry`]
//! inside a [`crate::LayerStack`].
//!
//! Layers are pure drawing primitives: the trait only exposes
//! z-order, name, and render. Per-layer state that the backend might
//! want to tweak at runtime — opacity, visibility, z-override, custom
//! name — lives on [`LayerEntry`], not on the trait, so the backend
//! can adjust them through the [`crate::LayerStack`] API without
//! downcasting.

use crate::framebuffer::FrameBuffer;

/// A unique handle for a layer inside a [`crate::LayerStack`].
///
/// Ids are assigned by the stack when a layer is pushed and remain
/// stable until the entry is removed. Ids are not reused within the
/// lifetime of a stack.
pub type LayerId = usize;

/// A single layer that can be drawn into a [`FrameBuffer`].
///
/// Implementations should be pure with respect to the rest of the
/// layer stack: the compositor handles ordering, visibility, and
/// opacity. A layer's [`Layer::render`] is expected to read the
/// destination's current state from `target` and write its
/// contribution blended at the given `opacity`.
pub trait Layer {
    /// The default z-order of this layer. Higher values are drawn
    /// later (on top); ties resolve by stack insertion order. The
    /// [`LayerEntry`] wrapper can override this with
    /// [`LayerEntry::set_z_override`].
    fn z_order(&self) -> u32;

    /// A human-readable name for the layer, used in error messages
    /// and debugging. The default returns a placeholder.
    fn name(&self) -> &str {
        "<unnamed layer>"
    }

    /// Renders this layer into `target`, alpha-blending with the
    /// destination pixels using `opacity` (in `0.0..=1.0`).
    /// Implementations must respect `opacity`: at `0.0` the target
    /// must be unchanged; at `1.0` the layer's own alpha determines
    /// the blend.
    fn render(&self, target: &mut FrameBuffer, opacity: f32);
}

/// A solid-colour layer: fills the entire target framebuffer with
/// one RGBA colour, alpha-blended using the layer's effective
/// opacity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolidColor {
    /// `[R, G, B, A]` in `0..=255` per channel.
    pub color: [u8; 4],
    z: u32,
    name: String,
}

impl SolidColor {
    /// Creates a new solid-color layer with the given RGBA channels.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: [r, g, b, a],
            z: 0,
            name: format!("SolidColor(r={r}, g={g}, b={b}, a={a})"),
        }
    }

    /// Builder: sets the default z-order. The override in
    /// [`LayerEntry`] (if any) wins.
    #[must_use]
    pub fn with_z(mut self, z: u32) -> Self {
        self.z = z;
        self
    }

    /// Builder: sets a human-readable name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl Layer for SolidColor {
    fn z_order(&self) -> u32 {
        self.z
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn render(&self, target: &mut FrameBuffer, opacity: f32) {
        let effective = (f32::from(self.color[3]) / 255.0 * opacity).clamp(0.0, 1.0);
        for pixel in target.pixels_mut() {
            crate::framebuffer::blend_over(pixel, &self.color, effective);
        }
    }
}

/// A [`Layer`] plus the per-entry control state managed by
/// [`crate::LayerStack`]: opacity, visibility, optional z-order
/// override, and an optional custom name.
pub struct LayerEntry {
    id: LayerId,
    layer: Box<dyn Layer>,
    opacity: f32,
    visible: bool,
    z_override: Option<u32>,
    name: Option<String>,
}

impl LayerEntry {
    /// Creates a new entry wrapping `layer` with the given `id`. The
    /// entry starts fully opaque, visible, with no z-override, and
    /// no custom name.
    pub fn new(id: LayerId, layer: Box<dyn Layer>) -> Self {
        Self {
            id,
            layer,
            opacity: 1.0,
            visible: true,
            z_override: None,
            name: None,
        }
    }

    /// Returns the entry's id.
    pub fn id(&self) -> LayerId {
        self.id
    }

    /// Returns a reference to the wrapped layer.
    pub fn layer(&self) -> &dyn Layer {
        &*self.layer
    }

    /// Returns the entry's opacity in `0.0..=1.0`.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Sets the entry's opacity, clamping to `0.0..=1.0`.
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Returns whether the entry is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggles the entry's visibility. Invisible entries are skipped
    /// by the compositor.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns the effective z-order used by the compositor: the
    /// override if set, otherwise the layer's default.
    pub fn effective_z(&self) -> u32 {
        self.z_override.unwrap_or_else(|| self.layer.z_order())
    }

    /// Sets an explicit z-order override, replacing any previous
    /// override. Pass to [`LayerEntry::clear_z_override`] to fall
    /// back to the layer's default.
    pub fn set_z_override(&mut self, z: u32) {
        self.z_override = Some(z);
    }

    /// Clears any z-order override; [`LayerEntry::effective_z`]
    /// falls back to the layer's default.
    pub fn clear_z_override(&mut self) {
        self.z_override = None;
    }

    /// Returns the entry's name: the override if set, otherwise the
    /// layer's [`Layer::name`].
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or_else(|| self.layer.name())
    }

    /// Sets a custom name for this entry.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Consumes the entry and returns the wrapped layer.
    pub fn into_layer_box(self) -> Box<dyn Layer> {
        self.layer
    }

    /// Replaces the wrapped layer, preserving the entry's id and
    /// control state. Useful for hot-swapping a layer's contents
    /// without invalidating external [`LayerId`] handles.
    pub fn set_layer(&mut self, layer: Box<dyn Layer>) {
        self.layer = layer;
    }
}

impl std::fmt::Debug for LayerEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayerEntry")
            .field("id", &self.id)
            .field("name", &self.name())
            .field("opacity", &self.opacity)
            .field("visible", &self.visible)
            .field("z_override", &self.z_override)
            .field("effective_z", &self.effective_z())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{Layer, LayerEntry, SolidColor};
    use crate::framebuffer::FrameBuffer;

    #[test]
    fn solid_color_default_z_is_zero() {
        let s = SolidColor::new(1, 2, 3, 4);
        assert_eq!(s.z_order(), 0);
        assert_eq!(s.color, [1, 2, 3, 4]);
    }

    #[test]
    fn solid_color_builders() {
        let s = SolidColor::new(1, 2, 3, 4).with_z(5).with_name("bg");
        assert_eq!(s.z_order(), 5);
        assert_eq!(s.name(), "bg");
    }

    #[test]
    fn solid_color_render_fills_with_color() {
        let s = SolidColor::new(10, 20, 30, 255);
        let mut fb = FrameBuffer::new(2, 2);
        s.render(&mut fb, 1.0);
        for px in fb.pixels() {
            assert_eq!(*px, [10, 20, 30, 255]);
        }
    }

    #[test]
    fn solid_color_render_zero_opacity_noop() {
        let s = SolidColor::new(10, 20, 30, 255);
        let mut fb = FrameBuffer::new(1, 1);
        s.render(&mut fb, 0.0);
        assert_eq!(fb.pixels()[0], [0, 0, 0, 0]);
    }

    #[test]
    fn layer_entry_opacity_clamps() {
        let e = LayerEntry::new(0, Box::new(SolidColor::new(0, 0, 0, 255)));
        let mut e = e;
        e.set_opacity(2.0);
        assert_eq!(e.opacity(), 1.0);
        e.set_opacity(-1.0);
        assert_eq!(e.opacity(), 0.0);
        e.set_opacity(0.5);
        assert_eq!(e.opacity(), 0.5);
    }

    #[test]
    fn layer_entry_visibility_toggle() {
        let mut e = LayerEntry::new(0, Box::new(SolidColor::new(0, 0, 0, 255)));
        assert!(e.is_visible());
        e.set_visible(false);
        assert!(!e.is_visible());
    }

    #[test]
    fn layer_entry_z_override_beats_layer_default() {
        let mut e = LayerEntry::new(0, Box::new(SolidColor::new(0, 0, 0, 255).with_z(2)));
        assert_eq!(e.effective_z(), 2);
        e.set_z_override(99);
        assert_eq!(e.effective_z(), 99);
        e.clear_z_override();
        assert_eq!(e.effective_z(), 2);
    }

    #[test]
    fn layer_entry_set_layer_keeps_id() {
        let mut e = LayerEntry::new(7, Box::new(SolidColor::new(1, 2, 3, 255)));
        let original_id = e.id();
        e.set_layer(Box::new(SolidColor::new(4, 5, 6, 255)));
        assert_eq!(e.id(), original_id);
        assert_eq!(e.layer().z_order(), 0);
    }

    #[test]
    fn layer_entry_debug_does_not_panic() {
        let e = LayerEntry::new(0, Box::new(SolidColor::new(0, 0, 0, 255).with_name("dbg")));
        let s = format!("{e:?}");
        assert!(s.contains("LayerEntry"));
        assert!(s.contains("dbg"));
    }

    #[test]
    fn layer_entry_name_override() {
        let mut e = LayerEntry::new(0, Box::new(SolidColor::new(0, 0, 0, 255).with_name("a")));
        assert_eq!(e.name(), "a");
        e.set_name("b");
        assert_eq!(e.name(), "b");
    }
}
