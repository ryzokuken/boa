use boa_gc::{Finalize, Trace};
use boa_macros::{utf16, JsData};
// use boa_profiler::Profiler;
use icu_decimal::provider::DecimalSymbolsV1Marker;
// use icu_list::provider::AndListV1Marker;
use icu_locid::{
    extensions::unicode::{key, Value},
    Locale,
};

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    realm::Realm,
    string::common::StaticJsStrings,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
};

use super::{
    locale::{canonicalize_locale_list, resolve_locale, validate_extension},
    options::{get_number_option, IntlOptions},
    Service,
};

mod options;
pub(crate) use options::*;

#[derive(Debug, Trace, Finalize, JsData)]
// Safety: `DurationFormat` only contains non-traceable types.
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct DurationFormat {
    locale: Locale,
    numbering_system: Option<Value>,
    style: GlobalStyle,
    fractional_digits: Option<i32>,
}

pub(super) struct DurationFormatLocaleOptions {
    numbering_system: Option<Value>,
}

impl Service for DurationFormat {
    // type LangMarker = icu_provider::impl_casting_upcast!(AndListV1Marker, DecimalSymbolsV1Marker);
    type LangMarker = DecimalSymbolsV1Marker;
    type LocaleOptions = DurationFormatLocaleOptions;

    fn resolve(
        locale: &mut Locale,
        options: &mut Self::LocaleOptions,
        provider: &crate::context::icu::IntlProvider,
    ) {
        let numbering_system = options
            .numbering_system
            .take()
            .filter(|nu| {
                validate_extension::<Self::LangMarker>(locale.id.clone(), key!("nu"), nu, provider)
            })
            .or_else(|| {
                locale
                    .extensions
                    .unicode
                    .keywords
                    .get(&key!("nu"))
                    .cloned()
                    .filter(|nu| {
                        validate_extension::<Self::LangMarker>(
                            locale.id.clone(),
                            key!("nu"),
                            nu,
                            provider,
                        )
                    })
            });
        locale.extensions.unicode.clear();

        if let Some(nu) = numbering_system.clone() {
            locale.extensions.unicode.keywords.set(key!("nu"), nu);
        }

        options.numbering_system = numbering_system
    }
}

impl IntrinsicObject for DurationFormat {
    fn init(realm: &Realm) {
        // TODO
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DurationFormat {
    const NAME: JsString = StaticJsStrings::DURATION_FORMAT;
}

impl BuiltInConstructor for DurationFormat {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::duration_format;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.DurationFormat` constructor without `new`")
                .into());
        }

        // 2. Let durationFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%DurationFormatPrototype%", « [[InitializedDurationFormat]], [[Locale]], [[DataLocale]], [[NumberingSystem]], [[Style]], [[YearsStyle]], [[YearsDisplay]], [[MonthsStyle]], [[MonthsDisplay]] , [[WeeksStyle]], [[WeeksDisplay]] , [[DaysStyle]], [[DaysDisplay]] , [[HoursStyle]], [[HoursDisplay]] , [[MinutesStyle]], [[MinutesDisplay]] , [[SecondsStyle]], [[SecondsDisplay]] , [[MillisecondsStyle]], [[MillisecondsDisplay]] , [[MicrosecondsStyle]], [[MicrosecondsDisplay]] , [[NanosecondsStyle]], [[NanosecondsDisplay]], [[FractionalDigits]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::duration_format,
            context,
        )?;

        // 3. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let locales = args.get_or_undefined(0);
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // 4. Let options be ? GetOptionsObject(options).
        let options = args.get_or_undefined(1);
        let options = get_options_object(options)?;

        // 5. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
        let matcher = get_option(&options, utf16!("localeMatcher"), context)?.unwrap_or_default();

        // 6. Let numberingSystem be ? GetOption(options, "numberingSystem", string, undefined, undefined).
        // 7. If numberingSystem is not undefined, then
        //     a. If numberingSystem does not match the Unicode Locale Identifier type nonterminal, throw a RangeError exception.
        let numbering_system = get_option(&options, utf16!("numberingSystem"), context)?;

        // 8. Let opt be the Record { [[localeMatcher]]: matcher, [[nu]]: numberingSystem }.
        let mut opt = IntlOptions {
            matcher,
            service_options: DurationFormatLocaleOptions { numbering_system },
        };

        // 9. Let r be ResolveLocale(%DurationFormat%.[[AvailableLocales]], requestedLocales, opt, %DurationFormat%.[[RelevantExtensionKeys]], %DurationFormat%.[[LocaleData]]).
        // 10. Let locale be r.[[locale]].
        let locale = resolve_locale::<Self>(&requested_locales, &mut opt, context.intl_provider());

        // 13. Let style be ? GetOption(options, "style", string, « "long", "short", "narrow", "digital" », "short").
        let style = GlobalStyle::from_options(&options, context);

        // 16. Let prevStyle be the empty String.
        let mut prev_style: String;

        // 17. For each row of Table 3, except the header row, in table order, do
        //     a. Let styleSlot be the Style Slot value of the current row.
        //     b. Let displaySlot be the Display Slot value of the current row.
        //     c. Let unit be the Unit value of the current row.
        //     d. Let valueList be the Values value of the current row.
        //     e. Let digitalBase be the Digital Default value of the current row.
        //     f. Let unitOptions be ? GetDurationUnitOptions(unit, options, style, valueList, digitalBase, prevStyle).
        //     g. Set the value of the styleSlot slot of durationFormat to unitOptions.[[Style]].
        //     h. Set the value of the displaySlot slot of durationFormat to unitOptions.[[Display]].
        //     i. If unit is one of "hours", "minutes", "seconds", "milliseconds", or "microseconds", then
        //         i. Set prevStyle to unitOptions.[[Style]].
        let unit_options = DurationUnitOptions::from_options(&options, context);

        // 18. Set durationFormat.[[FractionalDigits]] to ? GetNumberOption(options, "fractionalDigits", 0, 9, undefined).
        let fractional_digits =
            get_number_option(&options, utf16!("fractionalDigits"), 0, 9, context)?;

        // 11. Set durationFormat.[[Locale]] to locale.
        // 12. Set durationFormat.[[NumberingSystem]] to r.[[nu]].
        // 14. Set durationFormat.[[Style]] to style.
        // 15. Set durationFormat.[[DataLocale]] to r.[[dataLocale]].
        let duration_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self {
                locale,
                style,
                fractional_digits,
                numbering_system: opt.service_options.numbering_system,
            },
        );

        // 19. Return durationFormat.
        Ok(duration_format.into())
    }
}
