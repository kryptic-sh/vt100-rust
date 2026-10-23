use crate::term::BufWrite as _;
use alloc::string::String;

/// Represents a foreground or background color for cells.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
pub enum Color {
    /// The default terminal color.
    #[default]
    Default,

    /// An indexed terminal color.
    Idx(u8),

    /// An RGB terminal color. The parameters are (red, green, blue).
    Rgb(u8, u8, u8),
}

/// How text is underlined, as SGR 4 (4:1 to 4:5) and 21 set it.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
pub enum UnderlineStyle {
    /// Not underlined.
    #[default]
    None,
    /// A single line (SGR 4, 4:1).
    Single,
    /// A double line (SGR 21, 4:2).
    Double,
    /// A curly line (SGR 4:3).
    Curly,
    /// A dotted line (SGR 4:4).
    Dotted,
    /// A dashed line (SGR 4:5).
    Dashed,
}

impl UnderlineStyle {
    /// The style SGR 4:`n` sets; `None` for a number it does not name.
    #[must_use]
    pub fn from_sgr(n: u16) -> Option<Self> {
        Some(match n {
            0 => Self::None,
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Curly,
            4 => Self::Dotted,
            5 => Self::Dashed,
            _ => return None,
        })
    }

    /// Its number after SGR 4: (0 for none).
    #[must_use]
    pub fn sgr(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Single => 1,
            Self::Double => 2,
            Self::Curly => 3,
            Self::Dotted => 4,
            Self::Dashed => 5,
        }
    }
}

const TEXT_MODE_INTENSITY: u8 = 0b0000_0011;
const TEXT_MODE_BOLD: u8 = 0b0000_0001;
const TEXT_MODE_DIM: u8 = 0b0000_0010;
const TEXT_MODE_ITALIC: u8 = 0b0000_0100;
const TEXT_MODE_INVERSE: u8 = 0b0001_0000;
/// The underline style's number ([`UnderlineStyle::sgr`]), in the top bits.
const TEXT_MODE_UNDERLINE: u8 = 0b1110_0000;
const TEXT_MODE_UNDERLINE_SHIFT: u32 = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Attrs {
    pub fgcolor: Color,
    pub bgcolor: Color,
    /// The underline's colour (SGR 58), `Default` for the text's.
    pub ulcolor: Color,
    pub mode: u8,
}

impl Default for Attrs {
    fn default() -> Self {
        Self::new()
    }
}

impl Attrs {
    pub const fn new() -> Self {
        Self {
            fgcolor: Color::Default,
            bgcolor: Color::Default,
            ulcolor: Color::Default,
            mode: 0,
        }
    }

    pub fn bold(&self) -> bool {
        self.mode & TEXT_MODE_BOLD != 0
    }

    pub fn dim(&self) -> bool {
        self.mode & TEXT_MODE_DIM != 0
    }

    fn intensity(&self) -> u8 {
        self.mode & TEXT_MODE_INTENSITY
    }

    pub fn set_bold(&mut self) {
        self.mode &= !TEXT_MODE_INTENSITY;
        self.mode |= TEXT_MODE_BOLD;
    }

    pub fn set_dim(&mut self) {
        self.mode &= !TEXT_MODE_INTENSITY;
        self.mode |= TEXT_MODE_DIM;
    }

    pub fn set_normal_intensity(&mut self) {
        self.mode &= !TEXT_MODE_INTENSITY;
    }

    pub fn italic(&self) -> bool {
        self.mode & TEXT_MODE_ITALIC != 0
    }

    pub fn set_italic(&mut self, italic: bool) {
        if italic {
            self.mode |= TEXT_MODE_ITALIC;
        } else {
            self.mode &= !TEXT_MODE_ITALIC;
        }
    }

    pub fn underline(&self) -> bool {
        self.mode & TEXT_MODE_UNDERLINE != 0
    }

    pub fn set_underline(&mut self, underline: bool) {
        self.set_underline_style(if underline {
            UnderlineStyle::Single
        } else {
            UnderlineStyle::None
        });
    }

    pub fn underline_style(&self) -> UnderlineStyle {
        UnderlineStyle::from_sgr(u16::from(
            (self.mode & TEXT_MODE_UNDERLINE) >> TEXT_MODE_UNDERLINE_SHIFT,
        ))
        .unwrap_or_default()
    }

    pub fn set_underline_style(&mut self, style: UnderlineStyle) {
        self.mode &= !TEXT_MODE_UNDERLINE;
        self.mode |= style.sgr() << TEXT_MODE_UNDERLINE_SHIFT;
    }

    pub fn inverse(&self) -> bool {
        self.mode & TEXT_MODE_INVERSE != 0
    }

    pub fn set_inverse(&mut self, inverse: bool) {
        if inverse {
            self.mode |= TEXT_MODE_INVERSE;
        } else {
            self.mode &= !TEXT_MODE_INVERSE;
        }
    }

    pub fn write_escape_code_diff(
        &self,
        contents: &mut String,
        other: &Self,
    ) {
        self.write_escape_code_diff_fmt(contents, other)
            .expect("writing to a String cannot fail");
    }

    pub fn write_escape_code_diff_fmt(
        &self,
        writer: &mut impl alloc::fmt::Write,
        other: &Self,
    ) -> alloc::fmt::Result {
        if self != other && self == &Self::default() {
            return crate::term::ClearAttrs.write_fmt(writer);
        }

        let attrs = crate::term::Attrs::default();

        let attrs = if self.fgcolor == other.fgcolor {
            attrs
        } else {
            attrs.fgcolor(self.fgcolor)
        };
        let attrs = if self.bgcolor == other.bgcolor {
            attrs
        } else {
            attrs.bgcolor(self.bgcolor)
        };
        let attrs = if self.intensity() == other.intensity() {
            attrs
        } else {
            attrs.intensity(match self.intensity() {
                0 => crate::term::Intensity::Normal,
                TEXT_MODE_BOLD => crate::term::Intensity::Bold,
                TEXT_MODE_DIM => crate::term::Intensity::Dim,
                _ => unreachable!(),
            })
        };
        let attrs = if self.italic() == other.italic() {
            attrs
        } else {
            attrs.italic(self.italic())
        };
        let attrs = if self.underline_style() == other.underline_style() {
            attrs
        } else {
            attrs.underline_style(self.underline_style())
        };
        let attrs = if self.ulcolor == other.ulcolor {
            attrs
        } else {
            attrs.ulcolor(self.ulcolor)
        };
        let attrs = if self.inverse() == other.inverse() {
            attrs
        } else {
            attrs.inverse(self.inverse())
        };

        attrs.write_fmt(writer)
    }
}
