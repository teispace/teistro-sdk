//! The colour names `COLOR` may hold (RFC 7986 §5.9): the 147 CSS3
//! colour keywords of CSS Color Module Level 3, §4.3, each with the
//! sRGB value the same module gives it, which `X-APPLE-CALENDAR-COLOR`
//! needs since Apple's clients read a hexadecimal value.

use teistro_core::error::Error;

/// A CSS3 colour name, in lower case.
///
/// RFC 7986 §5.9 takes a name and nothing else, so a hexadecimal value is
/// refused here rather than written for a client to drop.
///
/// ```
/// use teistro_calendar::ical::Color;
///
/// let colour = Color::named("Turquoise")?;
/// assert_eq!((colour.name(), colour.hex().as_str()), ("turquoise", "#40E0D0"));
/// assert!(Color::named("#40E0D0").is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    name: &'static str,
    rgb: u32,
}

impl Color {
    /// The colour of this name, in any case.
    ///
    /// ```
    /// use teistro_calendar::ical::Color;
    ///
    /// assert_eq!(Color::named("GOLD")?, Color::named("gold")?);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for anything else, a hexadecimal value included,
    /// which RFC 7986 §5.9 does not allow.
    pub fn named(name: &str) -> Result<Color, Error> {
        let lower = name.to_ascii_lowercase();
        NAMES
            .binary_search_by_key(&lower.as_str(), |&(known, _)| known)
            .ok()
            .and_then(|at| NAMES.get(at))
            .map(|&(name, rgb)| Color { name, rgb })
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "{name:?} is not a CSS3 colour name, the only value RFC 7986 §5.9 allows"
                ))
            })
    }

    /// The name, in lower case, as `COLOR` writes it.
    ///
    /// ```
    /// use teistro_calendar::ical::Color;
    ///
    /// assert_eq!(Color::named("Gold")?.name(), "gold");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The sRGB value as `#RRGGBB`, as `X-APPLE-CALENDAR-COLOR` writes it.
    ///
    /// ```
    /// use teistro_calendar::ical::Color;
    ///
    /// assert_eq!(Color::named("navy")?.hex(), "#000080");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn hex(self) -> String {
        format!("#{:06X}", self.rgb)
    }
}

/// The keywords, sorted, each with its sRGB value.
const NAMES: [(&str, u32); 147] = [
    ("aliceblue", 0xF0_F8FF),
    ("antiquewhite", 0xFA_EBD7),
    ("aqua", 0x00_FFFF),
    ("aquamarine", 0x7F_FFD4),
    ("azure", 0xF0_FFFF),
    ("beige", 0xF5_F5DC),
    ("bisque", 0xFF_E4C4),
    ("black", 0x00_0000),
    ("blanchedalmond", 0xFF_EBCD),
    ("blue", 0x00_00FF),
    ("blueviolet", 0x8A_2BE2),
    ("brown", 0xA5_2A2A),
    ("burlywood", 0xDE_B887),
    ("cadetblue", 0x5F_9EA0),
    ("chartreuse", 0x7F_FF00),
    ("chocolate", 0xD2_691E),
    ("coral", 0xFF_7F50),
    ("cornflowerblue", 0x64_95ED),
    ("cornsilk", 0xFF_F8DC),
    ("crimson", 0xDC_143C),
    ("cyan", 0x00_FFFF),
    ("darkblue", 0x00_008B),
    ("darkcyan", 0x00_8B8B),
    ("darkgoldenrod", 0xB8_860B),
    ("darkgray", 0xA9_A9A9),
    ("darkgreen", 0x00_6400),
    ("darkgrey", 0xA9_A9A9),
    ("darkkhaki", 0xBD_B76B),
    ("darkmagenta", 0x8B_008B),
    ("darkolivegreen", 0x55_6B2F),
    ("darkorange", 0xFF_8C00),
    ("darkorchid", 0x99_32CC),
    ("darkred", 0x8B_0000),
    ("darksalmon", 0xE9_967A),
    ("darkseagreen", 0x8F_BC8F),
    ("darkslateblue", 0x48_3D8B),
    ("darkslategray", 0x2F_4F4F),
    ("darkslategrey", 0x2F_4F4F),
    ("darkturquoise", 0x00_CED1),
    ("darkviolet", 0x94_00D3),
    ("deeppink", 0xFF_1493),
    ("deepskyblue", 0x00_BFFF),
    ("dimgray", 0x69_6969),
    ("dimgrey", 0x69_6969),
    ("dodgerblue", 0x1E_90FF),
    ("firebrick", 0xB2_2222),
    ("floralwhite", 0xFF_FAF0),
    ("forestgreen", 0x22_8B22),
    ("fuchsia", 0xFF_00FF),
    ("gainsboro", 0xDC_DCDC),
    ("ghostwhite", 0xF8_F8FF),
    ("gold", 0xFF_D700),
    ("goldenrod", 0xDA_A520),
    ("gray", 0x80_8080),
    ("green", 0x00_8000),
    ("greenyellow", 0xAD_FF2F),
    ("grey", 0x80_8080),
    ("honeydew", 0xF0_FFF0),
    ("hotpink", 0xFF_69B4),
    ("indianred", 0xCD_5C5C),
    ("indigo", 0x4B_0082),
    ("ivory", 0xFF_FFF0),
    ("khaki", 0xF0_E68C),
    ("lavender", 0xE6_E6FA),
    ("lavenderblush", 0xFF_F0F5),
    ("lawngreen", 0x7C_FC00),
    ("lemonchiffon", 0xFF_FACD),
    ("lightblue", 0xAD_D8E6),
    ("lightcoral", 0xF0_8080),
    ("lightcyan", 0xE0_FFFF),
    ("lightgoldenrodyellow", 0xFA_FAD2),
    ("lightgray", 0xD3_D3D3),
    ("lightgreen", 0x90_EE90),
    ("lightgrey", 0xD3_D3D3),
    ("lightpink", 0xFF_B6C1),
    ("lightsalmon", 0xFF_A07A),
    ("lightseagreen", 0x20_B2AA),
    ("lightskyblue", 0x87_CEFA),
    ("lightslategray", 0x77_8899),
    ("lightslategrey", 0x77_8899),
    ("lightsteelblue", 0xB0_C4DE),
    ("lightyellow", 0xFF_FFE0),
    ("lime", 0x00_FF00),
    ("limegreen", 0x32_CD32),
    ("linen", 0xFA_F0E6),
    ("magenta", 0xFF_00FF),
    ("maroon", 0x80_0000),
    ("mediumaquamarine", 0x66_CDAA),
    ("mediumblue", 0x00_00CD),
    ("mediumorchid", 0xBA_55D3),
    ("mediumpurple", 0x93_70DB),
    ("mediumseagreen", 0x3C_B371),
    ("mediumslateblue", 0x7B_68EE),
    ("mediumspringgreen", 0x00_FA9A),
    ("mediumturquoise", 0x48_D1CC),
    ("mediumvioletred", 0xC7_1585),
    ("midnightblue", 0x19_1970),
    ("mintcream", 0xF5_FFFA),
    ("mistyrose", 0xFF_E4E1),
    ("moccasin", 0xFF_E4B5),
    ("navajowhite", 0xFF_DEAD),
    ("navy", 0x00_0080),
    ("oldlace", 0xFD_F5E6),
    ("olive", 0x80_8000),
    ("olivedrab", 0x6B_8E23),
    ("orange", 0xFF_A500),
    ("orangered", 0xFF_4500),
    ("orchid", 0xDA_70D6),
    ("palegoldenrod", 0xEE_E8AA),
    ("palegreen", 0x98_FB98),
    ("paleturquoise", 0xAF_EEEE),
    ("palevioletred", 0xDB_7093),
    ("papayawhip", 0xFF_EFD5),
    ("peachpuff", 0xFF_DAB9),
    ("peru", 0xCD_853F),
    ("pink", 0xFF_C0CB),
    ("plum", 0xDD_A0DD),
    ("powderblue", 0xB0_E0E6),
    ("purple", 0x80_0080),
    ("red", 0xFF_0000),
    ("rosybrown", 0xBC_8F8F),
    ("royalblue", 0x41_69E1),
    ("saddlebrown", 0x8B_4513),
    ("salmon", 0xFA_8072),
    ("sandybrown", 0xF4_A460),
    ("seagreen", 0x2E_8B57),
    ("seashell", 0xFF_F5EE),
    ("sienna", 0xA0_522D),
    ("silver", 0xC0_C0C0),
    ("skyblue", 0x87_CEEB),
    ("slateblue", 0x6A_5ACD),
    ("slategray", 0x70_8090),
    ("slategrey", 0x70_8090),
    ("snow", 0xFF_FAFA),
    ("springgreen", 0x00_FF7F),
    ("steelblue", 0x46_82B4),
    ("tan", 0xD2_B48C),
    ("teal", 0x00_8080),
    ("thistle", 0xD8_BFD8),
    ("tomato", 0xFF_6347),
    ("turquoise", 0x40_E0D0),
    ("violet", 0xEE_82EE),
    ("wheat", 0xF5_DEB3),
    ("white", 0xFF_FFFF),
    ("whitesmoke", 0xF5_F5F5),
    ("yellow", 0xFF_FF00),
    ("yellowgreen", 0x9A_CD32),
];

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn the_names_are_sorted_so_a_search_finds_every_one() {
        assert!(NAMES.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (name, rgb) in NAMES {
            assert_eq!(Color::named(name).unwrap(), Color { name, rgb });
        }
    }

    #[test]
    fn a_grey_is_spelt_both_ways_and_a_later_name_is_refused() {
        assert_eq!(
            Color::named("gray").unwrap().hex(),
            Color::named("grey").unwrap().hex()
        );
        // CSS4's addition: RFC 7986 names CSS3.
        assert!(Color::named("rebeccapurple").is_err());
    }
}
