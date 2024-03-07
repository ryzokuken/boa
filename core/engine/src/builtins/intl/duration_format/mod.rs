use boa_macros::utf16;
use boa_profiler::Profiler;
use icu_list::provider::AndListV1Marker;
use icu_locid::{extensions::unicode::{key, Value}, Locale};

use crate::{builtins::{number, options::{get_option, get_options_object}, BuiltInConstructor, BuiltInObject, IntrinsicObject}, context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}, realm::Realm, string::common::StaticJsStrings, Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue};

use super::{locale::{canonicalize_locale_list, resolve_locale, validate_extension}, number_format::NumberFormatLocaleOptions, options::IntlOptions, Service};

pub(crate) struct DurationFormat {
    locale: Locale,
    numbering_system: Option<Value>,
    style: JsString,
    fractional_digits: i32,
}

impl DurationFormat {

}

pub(super) struct DurationFormatLocaleOptions {
    numbering_system: Option<Value>,
}

impl Service for DurationFormat {
    // type LangMarker = icu_provider::impl_casting_upcast!(AndListV1Marker, DecimalSymbolsV1Marker);
    type LangMarker = AndListV1Marker;
    type LocaleOptions = DurationFormatLocaleOptions;

    fn resolve(locale: &mut Locale, options: &mut Self::LocaleOptions, provider: &crate::context::icu::IntlProvider) {
        let numbering_system = options.numbering_system.take().filter(|nu| {
            validate_extension::<Self::LangMarker>(locale.id.clone(), key!("nu"), nu, provider)
        }).or_else(|| {
            locale.extensions.unicode.keywords.get(&key!("nu")).cloned().filter(|nu| {
                validate_extension::<Self::LangMarker>(locale.id.clone(), key!("nu"), nu, provider)
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
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        // more
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

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor = StandardConstructors::duration_format;

    fn constructor(new_target: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ().with_message("cannot call `Intl.DurationFormat` constructor without `new`").into());
        }

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
        let opt = IntlOptions {
            matcher,
            service_options: DurationFormatLocaleOptions { numbering_system },
        };

        // 9. Let r be ResolveLocale(%DurationFormat%.[[AvailableLocales]], requestedLocales, opt, %DurationFormat%.[[RelevantExtensionKeys]], %DurationFormat%.[[LocaleData]]).
        let r = resolve_locale::<Self>(&requested_locales, &mut opt, context.intl_provider());

        // 2. Let durationFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%DurationFormatPrototype%", « [[InitializedDurationFormat]], [[Locale]], [[DataLocale]], [[NumberingSystem]], [[Style]], [[YearsStyle]], [[YearsDisplay]], [[MonthsStyle]], [[MonthsDisplay]] , [[WeeksStyle]], [[WeeksDisplay]] , [[DaysStyle]], [[DaysDisplay]] , [[HoursStyle]], [[HoursDisplay]] , [[MinutesStyle]], [[MinutesDisplay]] , [[SecondsStyle]], [[SecondsDisplay]] , [[MillisecondsStyle]], [[MillisecondsDisplay]] , [[MicrosecondsStyle]], [[MicrosecondsDisplay]] , [[NanosecondsStyle]], [[NanosecondsDisplay]], [[FractionalDigits]] »).
        let duration_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self {
                locale,
                style,
                fractional_digits
            },
        );

        // 19. Return durationFormat.
        Ok(duration_format.into())
    }
}
