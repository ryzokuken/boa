use std::str::FromStr;

use boa_macros::utf16;

use crate::{
    builtins::options::{get_option, ParsableOptionType},
    Context, JsNativeError, JsObject, JsResult,
};

#[derive(Default, Debug, PartialEq)]
pub(crate) enum GlobalStyle {
    Long,
    #[default]
    Short,
    Narrow,
    Digital,
}

pub(crate) struct ParseGlobalStyleError;

impl std::fmt::Display for ParseGlobalStyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid style option")
    }
}

impl FromStr for GlobalStyle {
    type Err = ParseGlobalStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            "narrow" => Ok(Self::Narrow),
            "digital" => Ok(Self::Digital),
            _ => Err(ParseGlobalStyleError),
        }
    }
}

impl ParsableOptionType for GlobalStyle {}

impl GlobalStyle {
    pub(crate) fn from_options(options: &JsObject, context: &mut Context) -> Self {
        get_option(options, utf16!("style"), context)
            .unwrap_or_default()
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Style {
    Long,
    Short,
    Narrow,
    Numeric,
    TwoDigit,
    Fractional,
}

struct ParseStyleError;

impl std::fmt::Display for ParseStyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid style option")
    }
}

impl FromStr for Style {
    type Err = ParseStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            "narrow" => Ok(Self::Narrow),
            "numeric" => Ok(Self::Numeric),
            "2-digit" => Ok(Self::TwoDigit),
            "fractional" => Ok(Self::Fractional),
            _ => Err(ParseStyleError),
        }
    }
}

impl ParsableOptionType for Style {}

enum StylesList {
    Base,
    Digital,
    Fractional,
}

impl StylesList {
    fn list(self) -> Vec<Style> {
        match self {
            Self::Base => base_styles().collect(),
            Self::Digital => digital_styles().collect(),
            Self::Fractional => fractional_styles().collect(),
        }
    }
}

fn base_styles() -> impl Iterator<Item = Style> {
    [Style::Long, Style::Short, Style::Short].iter().copied()
}

fn digital_styles() -> impl Iterator<Item = Style> {
    base_styles().chain([Style::Numeric, Style::TwoDigit])
}

fn fractional_styles() -> impl Iterator<Item = Style> {
    base_styles().chain([Style::Fractional])
}

#[derive(PartialEq)]
enum Display {
    Auto,
    Always,
}

struct ParseDisplayError;

impl std::fmt::Display for ParseDisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid display option")
    }
}

impl FromStr for Display {
    type Err = ParseDisplayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            _ => Err(ParseDisplayError),
        }
    }
}

impl ParsableOptionType for Display {}

#[derive(PartialEq)]
pub(crate) enum Unit {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

impl Unit {
    fn to_str(self) -> &'static str {
        match self {
            Years => "years",
            Months => "months",
            Weeks => "weeks",
            Days => "days",
            Hours => "hours",
            Minutes => "minutes",
            Seconds => "seconds",
            Milliseconds => "milliseconds",
            Microseconds => "microseconds",
            Nanoseconds => "nanoseconds",
        }
    }

    fn styles_list(self) -> StylesList {
        if [Self::Years, Self::Months, Self::Weeks, Self::Days].contains(&self) {
            StylesList::Base
        } else if [Self::Hours, Self::Minutes, Self::Seconds].contains(&self) {
            StylesList::Digital
        } else {
            StylesList::Fractional
        }
    }

    fn digital_default(self) -> Style {
        if [Self::Years, Self::Months, Self::Weeks, Self::Days].contains(&self) {
            Style::Short
        } else if [Self::Microseconds, Self::Nanoseconds].contains(&self) {
            Style::Fractional
        } else {
            Style::Numeric
        }
    }

    fn style_from_options(
        self,
        options: &JsObject,
        context: &mut Context,
    ) -> JsResult<Option<Style>> {
        let key = self.to_str();
        let key_utf16: Vec<u16> = key.encode_utf16().collect();
        let list = self.styles_list().list();
        let style = get_option(options, &key_utf16, context)?;
        if let Some(s) = &style {
            if !list.contains(s) {
                return Err(JsNativeError::range()
                    .with_message(format!("Invalid style value for unit: {}", key))
                    .into());
            }
        }
        Ok(style)
    }

    fn display_from_options(
        self,
        options: &JsObject,
        default: Display,
        context: &mut Context,
    ) -> JsResult<Display> {
        let key: Vec<u16> = self
            .to_str()
            .encode_utf16()
            .chain(utf16!("Display").iter().copied())
            .collect();
        let display = get_option(options, &key, context)?;
        Ok(display.unwrap_or(default))
    }
}

struct UnitOptions {
    style: Style,
    display: Display,
}

impl UnitOptions {
    /// Abstract operation [`GetDurationUnitOptions ( unit, options, baseStyle, stylesList, digitalBase, prevStyle )`][spec]
    ///
    /// [spec]: https://tc39.es/proposal-intl-duration-format/#sec-getdurationunitoptions
    fn from_options(
        unit: Unit,
        options: &JsObject,
        base_style: GlobalStyle,
        prev_style: Style,
        context: &mut Context,
    ) -> JsResult<Self> {
        // 1. Let style be ? GetOption(options, unit, string, stylesList, undefined).
        let mut style = unit.style_from_options(options, context)?;

        // 2. Let displayDefault be "always".
        let mut display_default = Display::Always;

        // 3. If style is undefined, then
        if style.is_none() {
            // a. If baseStyle is "digital", then
            if base_style == GlobalStyle::Digital {
                // i. If unit is not one of "hours", "minutes", or "seconds", then
                if ![Unit::Hours, Unit::Minutes, Unit::Seconds].contains(&unit) {
                    // 1. Set displayDefault to "auto".
                    display_default = Display::Auto;
                }
                // ii. Set style to digitalBase.
                style = Some(unit.digital_default());
            // b. Else,
            } else {
                // i. If prevStyle is "fractional", "numeric" or "2-digit", then
                if [Style::Fractional, Style::Numeric, Style::TwoDigit].contains(&prev_style) {
                    // 1. If unit is not one of "minutes" or "seconds", then
                    if ![Unit::Minutes, Unit::Seconds].contains(&unit) {
                        // a. Set displayDefault to "auto".
                        display_default = Display::Auto;
                    }
                    // 2. Set style to "numeric".
                    style = Some(Style::Numeric);
                // ii. Else,
                } else {
                    // 1. Set displayDefault to "auto".
                    display_default = Display::Auto;
                    // 2. Set style to baseStyle.
                    style = Some(base_style.into()); // TODO: Fix this
                }
            }
        }
        let mut style = style.unwrap();

        // 4. If style is "numeric", then
        if style == Style::Numeric {
            // a. If unit is one of "milliseconds", "microseconds", or "nanoseconds", then
            if [Unit::Milliseconds, Unit::Microseconds, Unit::Nanoseconds].contains(&unit) {
                // i. Set style to "fractional".
                style = Style::Fractional;
                // ii. Set displayDefault to "auto".
                display_default = Display::Auto;
            }
        }

        // 5. Let displayField be the string-concatenation of unit and "Display".
        // 6. Let display be ? GetOption(options, displayField, string, « "auto", "always" », displayDefault).
        let display = unit.display_from_options(options, display_default, context)?;

        let range_error = Err(JsNativeError::range()
            .with_message("incompatible options provided")
            .into());

        // 7. If display is "always" and style is "fractional", then
        if display == Display::Always && style == Style::Fractional {
            // a. Throw a RangeError exception.
            return range_error;
        }

        // 8. If prevStyle is "fractional", then
        if prev_style == Style::Fractional {
            // a. If style is not "fractional", then
            if style != Style::Fractional {
                // i. Throw a RangeError exception.
                return range_error;
            }
        }

        // 9. If prevStyle is "numeric" or "2-digit", then
        if [Style::Numeric, Style::TwoDigit].contains(&prev_style) {
            // a. If style is not "fractional", "numeric" or "2-digit", then
            if ![Style::Fractional, Style::Numeric, Style::TwoDigit].contains(&style) {
                // i. Throw a RangeError exception.
                return range_error;
            }
            // b. If unit is "minutes" or "seconds", then
            if [Unit::Minutes, Unit::Seconds].contains(&unit) {
                // i. Set style to "2-digit".
                style = Style::TwoDigit;
            }
        }

        // 10. Return the Record { [[Style]]: style, [[Display]]: display  }.
        Ok(Self { style, display })
    }
}

// pub(super) struct DurationUnitOptions {
//     years: UnitOptions,
//     months: UnitOptions,
//     weeks: UnitOptions,
//     days: UnitOptions,
//     hours: UnitOptions,
//     minutes: UnitOptions,
//     seconds: UnitOptions,
//     milliseconds: UnitOptions,
//     microseconds: UnitOptions,
//     nanoseconds: UnitOptions,
// }

// impl DurationUnitOptions {
//     pub(super) fn from_options(options: &JsObject, context: &mut Context) -> Self {
//         Self {}
//     }
// }
