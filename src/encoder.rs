//! Output protocol selection and framebuffer encoding.
//!
//! The runtime picks a [`Protocol`] (Kitty graphics protocol or
//! Sixel) based on terminal capability detection (via `TERM`,
//! `TERM_PROGRAM`, `COLORTERM`) per AGENTS.md §7, preferring
//! [`Protocol::Kitty`] when the host supports it and falling back
//! to [`Protocol::Sixel`] otherwise.
//!
//! v0.5.0 wires up the Kitty arm via the optional
//! [`little_kitty`](https://crates.io/crates/little-kitty) crate
//! behind the `kitty-encoder` Cargo feature. v0.6.0 wires up the
//! Sixel arm via the optional
//! [`icy_sixel`](https://crates.io/crates/icy_sixel) crate behind
//! the `sixel-encoder` Cargo feature. Each arm returns
//! [`EncoderError::UnsupportedProtocol`] when the corresponding
//! feature is disabled in the current build.

use crate::framebuffer::FrameBuffer;

/// Terminal graphics protocol used to encode the composited
/// framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// The kitty graphics protocol -- modern and feature-rich.
    Kitty,
    /// Sixel -- fallback for terminals without kitty support.
    Sixel,
}

impl Protocol {
    /// Returns the protocol name as it appears in docs and
    /// capability probes.
    pub const fn as_str(self) -> &'static str {
        match self {
            Protocol::Kitty => "kitty",
            Protocol::Sixel => "sixel",
        }
    }
}

/// Errors produced by [`ProtocolEncoder::encode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncoderError {
    /// The requested protocol is not compiled into this build
    /// (e.g. calling `encode` on `Protocol::Kitty` without the
    /// `kitty-encoder` feature, or on `Protocol::Sixel` without
    /// the `sixel-encoder` feature).
    UnsupportedProtocol(&'static str),

    /// The framebuffer has zero width or height and cannot be
    /// encoded.
    InvalidDimensions {
        /// Framebuffer width in pixels.
        width: u32,
        /// Framebuffer height in pixels.
        height: u32,
    },

    /// The underlying encoder crate failed; the wrapped `String`
    /// carries its `Display` output.
    Encode(String),
}

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedProtocol(p) => {
                write!(f, "protocol {p} is not supported in this build")
            }
            Self::InvalidDimensions { width, height } => {
                write!(f, "framebuffer has invalid dimensions: {width}x{height}")
            }
            Self::Encode(msg) => write!(f, "encoder failed: {msg}"),
        }
    }
}

impl std::error::Error for EncoderError {}

// `From` impls for the underlying encoder-crate error types.
// Gated on the respective features so a build that doesn't pull
// in the crate can't reference its error type. The shared shape
// `EncoderError::Encode(String)` lets the per-encoder `encode`
// functions use `?` directly without per-module helper closures
// (the v0.5.0 `io_err` / v0.6.0 `sixel_err` helpers have been
// removed in favour of this pattern).

#[cfg(feature = "kitty-encoder")]
impl From<std::io::Error> for EncoderError {
    fn from(e: std::io::Error) -> Self {
        EncoderError::Encode(e.to_string())
    }
}

#[cfg(feature = "sixel-encoder")]
impl From<icy_sixel::SixelError> for EncoderError {
    fn from(e: icy_sixel::SixelError) -> Self {
        EncoderError::Encode(e.to_string())
    }
}

/// Encodes a [`FrameBuffer`] into the byte stream a terminal
/// expects for a chosen [`Protocol`].
///
/// Implementors return a `Vec<u8>` of escape sequences the caller
/// writes to stdout; the encoding does no I/O itself.
pub trait ProtocolEncoder {
    /// Encodes `frame` into escape-sequence bytes for `self`.
    fn encode(&self, frame: &FrameBuffer) -> Result<Vec<u8>, EncoderError>;
}

impl ProtocolEncoder for Protocol {
    fn encode(&self, frame: &FrameBuffer) -> Result<Vec<u8>, EncoderError> {
        match self {
            #[cfg(feature = "kitty-encoder")]
            Protocol::Kitty => kitty::encode(frame),
            #[cfg(not(feature = "kitty-encoder"))]
            Protocol::Kitty => {
                let _ = frame;
                Err(EncoderError::UnsupportedProtocol("kitty"))
            },
            #[cfg(feature = "sixel-encoder")]
            Protocol::Sixel => sixel::encode(frame),
            #[cfg(not(feature = "sixel-encoder"))]
            Protocol::Sixel => {
                let _ = frame;
                Err(EncoderError::UnsupportedProtocol("sixel"))
            },
        }
    }
}

/// The Kitty graphics protocol encoder, gated on the
/// `kitty-encoder` Cargo feature. Implemented as a private inline
/// module so the public API surface stays minimal.
#[cfg(feature = "kitty-encoder")]
mod kitty {
    use super::EncoderError;
    use crate::framebuffer::FrameBuffer;
    use little_kitty::command::ControlValue;
    use little_kitty::io::KittyCommandWriter;
    use std::io::Write;

    /// Encodes `frame` as a single Kitty "transmit and display"
    /// command using raw RGBA pixel data (format code 32 per the
    /// Kitty graphics protocol spec). The returned bytes are the
    /// full escape-sequence payload ready to be written to the
    /// terminal.
    pub fn encode(frame: &FrameBuffer) -> Result<Vec<u8>, EncoderError> {
        if frame.width() == 0 || frame.height() == 0 {
            return Err(EncoderError::InvalidDimensions {
                width: frame.width(),
                height: frame.height(),
            });
        }

        // Materialise the RGBA pixel data as a single contiguous
        // byte slice. A streaming encode can be added later if
        // the per-frame allocation becomes a hotspot.
        let rgba: Vec<u8> = frame.pixels().iter().flatten().copied().collect();

        // Build the control list. The Kitty graphics protocol
        // accepts a comma-separated list of key=value pairs
        // before the payload separator (`;`). We use:
        //   a=T   -- action: transmit and put (display)
        //   f=32  -- format: 32-bit RGBA
        //   q=2   -- quiet: suppress terminal OK/error responses
        //   s=W   -- image width in pixels
        //   v=H   -- image height in pixels
        let controls: Vec<(char, ControlValue)> = vec![
            ('a', ControlValue::Char('T')),
            ('f', ControlValue::UnsignedInteger(32)),
            ('q', ControlValue::UnsignedInteger(2)),
            ('s', ControlValue::UnsignedInteger(frame.width())),
            ('v', ControlValue::UnsignedInteger(frame.height())),
        ];

        let mut out = Vec::new();
        out.write_start(false, None)?;
        for (i, (key, value)) in controls.iter().enumerate() {
            if i > 0 {
                out.write_all(b",")?;
            }
            write!(out, "{key}=")?;
            value.write(&mut out)?;
        }
        out.write_all(b";")?;
        // write_base64 consumes the writer by value (returns Self)
        // and Base64-encodes the payload per the Kitty graphics protocol.
        // TODO(v0.5.x): chunk large images (m=0 more-chunks / m=1 last chunk)
        // to support multi-megapixel framebuffers; the current single-command
        // encoder will hit terminal size limits for very large frames.
        out = out.write_base64(&rgba)?;
        out.write_end(false)?;
        Ok(out)
    }
}

/// The Sixel graphics protocol encoder, gated on the
/// `sixel-encoder` Cargo feature. Implemented as a private inline
/// module so the public API surface stays minimal.
#[cfg(feature = "sixel-encoder")]
mod sixel {
    use super::EncoderError;
    use crate::framebuffer::FrameBuffer;
    use icy_sixel::SixelImage;

    /// Encodes `frame` as a Sixel DCS (Device Control String)
    /// escape sequence. The returned bytes are the full
    /// terminal-ready payload: `\x1bPq...sixel data...\x1b\\`.
    /// `icy_sixel` does the color quantization and sixel-data
    /// serialisation; we just hand it the RGBA pixels and pass
    /// through the resulting string.
    pub fn encode(frame: &FrameBuffer) -> Result<Vec<u8>, EncoderError> {
        if frame.width() == 0 || frame.height() == 0 {
            return Err(EncoderError::InvalidDimensions {
                width: frame.width(),
                height: frame.height(),
            });
        }

        // Materialise the RGBA pixel data as a single contiguous
        // byte slice. `icy_sixel` takes owned bytes.
        let rgba: Vec<u8> = frame.pixels().iter().flatten().copied().collect();

        // `SixelImage::from_rgba` takes `usize` width/height; the
        // `u32` values from FrameBuffer are always representable
        // in `usize` on every supported platform (a widening,
        // lossless cast).
        let image = SixelImage::from_rgba(rgba, frame.width() as usize, frame.height() as usize);
        let sixel_string = image.encode()?;
        Ok(sixel_string.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::{EncoderError, Protocol, ProtocolEncoder};
    use crate::framebuffer::FrameBuffer;

    #[test]
    fn as_str_matches_variant() {
        assert_eq!(Protocol::Kitty.as_str(), "kitty");
        assert_eq!(Protocol::Sixel.as_str(), "sixel");
    }

    #[test]
    fn encoder_error_display_includes_context() {
        let e = EncoderError::UnsupportedProtocol("sixel");
        assert_eq!(e.to_string(), "protocol sixel is not supported in this build");

        let e = EncoderError::InvalidDimensions { width: 0, height: 5 };
        assert_eq!(e.to_string(), "framebuffer has invalid dimensions: 0x5");
    }

    #[cfg(not(feature = "sixel-encoder"))]
    #[test]
    fn sixel_encode_is_unsupported_without_feature() {
        // When the sixel-encoder feature is off, the Sixel arm
        // must return UnsupportedProtocol instead of producing
        // a (non-existent) encoder.
        let fb = FrameBuffer::new(2, 2);
        let err = Protocol::Sixel.encode(&fb).unwrap_err();
        assert_eq!(err, EncoderError::UnsupportedProtocol("sixel"));
    }

    #[cfg(not(feature = "kitty-encoder"))]
    #[test]
    fn kitty_encode_is_unsupported_without_feature() {
        // When the kitty-encoder feature is off, the Kitty arm
        // must return UnsupportedProtocol instead of producing
        // a (non-existent) encoder.
        let fb = FrameBuffer::new(2, 2);
        let err = Protocol::Kitty.encode(&fb).unwrap_err();
        assert_eq!(err, EncoderError::UnsupportedProtocol("kitty"));
    }

    #[cfg(feature = "kitty-encoder")]
    #[test]
    fn kitty_encode_rejects_zero_dimensions() {
        let fb_zero_w = FrameBuffer::new(0, 5);
        let fb_zero_h = FrameBuffer::new(5, 0);
        let fb_zero_both = FrameBuffer::new(0, 0);
        for fb in [&fb_zero_w, &fb_zero_h, &fb_zero_both] {
            let err = Protocol::Kitty.encode(fb).unwrap_err();
            assert!(matches!(err, EncoderError::InvalidDimensions { .. }));
        }
    }

    #[cfg(feature = "kitty-encoder")]
    #[test]
    fn kitty_encode_produces_valid_escape_framing() {
        // 2x2 fully-opaque red framebuffer.
        let mut fb = FrameBuffer::new(2, 2);
        for px in fb.pixels_mut() {
            *px = [255, 0, 0, 255];
        }
        let bytes = Protocol::Kitty.encode(&fb).unwrap();
        assert!(!bytes.is_empty(), "encoded output must not be empty");
        // The Kitty graphics protocol APC starts with ESC _G and
        // ends with ESC \\. See
        // https://sw.kovidgoyal.net/kitty/graphics-protocol/
        assert!(
            bytes.starts_with(b"\x1b_G"),
            "encoded output must start with the Kitty APC start (\\x1b_G), got: {:?}",
            &bytes[..bytes.len().min(8)],
        );
        assert!(
            bytes.ends_with(b"\x1b\\"),
            "encoded output must end with the Kitty APC terminator (\\x1b\\\\), got tail: {:?}",
            &bytes[bytes.len().saturating_sub(8)..],
        );
        // Decode the control payload (between the APC start and
        // the `;` separator) as UTF-8 and verify it contains the
        // expected keys and values for a 2x2 32-bit RGBA
        // transmit-and-display command.
        let s = std::str::from_utf8(&bytes).unwrap_or("");
        let payload_start = "\x1b_G".len();
        let payload_end = s.find(';').unwrap_or(s.len());
        let controls = &s[payload_start..payload_end];
        assert!(
            controls.contains("a=T"),
            "controls must include `a=T` (transmit and put), got: {controls:?}",
        );
        assert!(
            controls.contains("f=32"),
            "controls must include `f=32` (32-bit RGBA), got: {controls:?}",
        );
        assert!(
            controls.contains("q=2"),
            "controls must include `q=2` (suppress responses), got: {controls:?}",
        );
        assert!(
            controls.contains("s=2"),
            "controls must include `s=2` (width 2), got: {controls:?}",
        );
        assert!(
            controls.contains("v=2"),
            "controls must include `v=2` (height 2), got: {controls:?}",
        );
    }

    #[cfg(feature = "kitty-encoder")]
    #[test]
    fn kitty_encode_is_deterministic_for_same_input() {
        // Two calls with the same input must produce identical
        // bytes (the encoder is pure with respect to the frame).
        let mut fb = FrameBuffer::new(3, 3);
        for px in fb.pixels_mut() {
            *px = [10, 20, 30, 255];
        }
        let a = Protocol::Kitty.encode(&fb).unwrap();
        let b = Protocol::Kitty.encode(&fb).unwrap();
        assert_eq!(a, b);
    }

    #[cfg(feature = "sixel-encoder")]
    #[test]
    fn sixel_encode_rejects_zero_dimensions() {
        let fb_zero_w = FrameBuffer::new(0, 5);
        let fb_zero_h = FrameBuffer::new(5, 0);
        let fb_zero_both = FrameBuffer::new(0, 0);
        for fb in [&fb_zero_w, &fb_zero_h, &fb_zero_both] {
            let err = Protocol::Sixel.encode(fb).unwrap_err();
            assert!(matches!(err, EncoderError::InvalidDimensions { .. }));
        }
    }

    #[cfg(feature = "sixel-encoder")]
    #[test]
    fn sixel_encode_produces_valid_dcs_framing() {
        // 2x2 fully-opaque red framebuffer.
        let mut fb = FrameBuffer::new(2, 2);
        for px in fb.pixels_mut() {
            *px = [255, 0, 0, 255];
        }
        let bytes = Protocol::Sixel.encode(&fb).unwrap();
        assert!(!bytes.is_empty(), "encoded output must not be empty");
        // Sixel is wrapped in a DCS (Device Control String)
        // introducer `\x1bP` and ends with the ST (String
        // Terminator) `\x1b\\`. See
        // https://en.wikipedia.org/wiki/Sixel
        assert!(
            bytes.starts_with(b"\x1bP"),
            "encoded output must start with the Sixel DCS introducer (\\x1bP), got: {:?}",
            &bytes[..bytes.len().min(8)],
        );
        assert!(
            bytes.ends_with(b"\x1b\\"),
            "encoded output must end with the DCS ST terminator (\\x1b\\\\), got tail: {:?}",
            &bytes[bytes.len().saturating_sub(8)..],
        );
        // The Sixel format-string mode letter `q` must appear
        // somewhere between the DCS introducer and the first
        // colour-definition introducer `#`. The exact position
        // may vary if pixel-aspect-ratio parameters are present,
        // so we just check the ordering rather than an exact
        // prefix.
        let header_end = bytes.iter().position(|&b| b == b'#').unwrap_or(bytes.len());
        let header = &bytes[..header_end];
        assert!(
            header.contains(&b'q'),
            "Sixel header must contain the `q` mode letter before the first `#`, got: {:?}",
            std::str::from_utf8(header).unwrap_or("<non-utf8>"),
        );
        // Best-effort: the output should be non-trivially longer
        // than just the 4-byte framing (`\x1bP` + `\x1b\\`), to
        // catch a regression where the encoder emits no actual
        // pixel data.
        assert!(
            bytes.len() > 16,
            "encoded output must contain real pixel data (got {} bytes)",
            bytes.len(),
        );
        // Best-effort: the payload should mention the 2x2
        // dimensions somewhere in the raster attributes or data.
        let s = std::str::from_utf8(&bytes).unwrap_or("");
        assert!(
            s.contains('2'),
            "encoded output should reference the 2x2 dimensions, got: {s:?}",
        );
    }

    #[cfg(feature = "sixel-encoder")]
    #[test]
    fn sixel_encode_is_deterministic_for_same_input() {
        // Two calls with the same input must produce identical
        // bytes (the encoder is pure with respect to the frame).
        let mut fb = FrameBuffer::new(3, 3);
        for px in fb.pixels_mut() {
            *px = [10, 20, 30, 255];
        }
        let a = Protocol::Sixel.encode(&fb).unwrap();
        let b = Protocol::Sixel.encode(&fb).unwrap();
        assert_eq!(a, b);
    }
}
