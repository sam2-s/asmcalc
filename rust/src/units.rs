//! Unit conversion.
//!
//! Units are stored as a factor to a base unit, so a conversion is a single
//! division. Only linear units are here: temperature is the exception, because
//! Celsius and Fahrenheit have offsets as well as factors, and it gets its own
//! path rather than pretending a factor is enough.

/// A category of convertible quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Length,
    Mass,
    Temperature,
    Area,
    Volume,
    Time,
    Speed,
    Data,
}

impl Category {
    pub fn from_name(name: &str) -> Option<Category> {
        Some(match name.to_ascii_lowercase().as_str() {
            "length" => Category::Length,
            "mass" | "weight" => Category::Mass,
            "temperature" | "temp" => Category::Temperature,
            "area" => Category::Area,
            "volume" => Category::Volume,
            "time" => Category::Time,
            "speed" => Category::Speed,
            "data" => Category::Data,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Category::Length => "Length",
            Category::Mass => "Mass",
            Category::Temperature => "Temperature",
            Category::Area => "Area",
            Category::Volume => "Volume",
            Category::Time => "Time",
            Category::Speed => "Speed",
            Category::Data => "Data",
        }
    }

    pub fn units(self) -> &'static [(&'static str, f64)] {
        match self {
            // Metres, kilograms, and so on; the first unit is the base.
            Category::Length => &[
                ("mm", 0.001),
                ("cm", 0.01),
                ("m", 1.0),
                ("km", 1000.0),
                ("in", 0.0254),
                ("ft", 0.3048),
                ("yd", 0.9144),
                ("mi", 1609.344),
                ("mile", 1609.344),
                ("miles", 1609.344),
                ("nmi", 1852.0),
            ],
            Category::Mass => &[
                ("mg", 1e-6),
                ("g", 0.001),
                ("kg", 1.0),
                ("t", 1000.0),
                ("oz", 0.028_349_523_125),
                ("lb", 0.453_592_37),
                ("lbs", 0.453_592_37),
                ("st", 6.350_293_18),
            ],
            // Offsets are handled in convert, so the factors here are 1.
            Category::Temperature => &[("C", 1.0), ("F", 1.0), ("K", 1.0)],
            Category::Area => &[
                ("mm2", 1e-6),
                ("cm2", 1e-4),
                ("m2", 1.0),
                ("ha", 10_000.0),
                ("km2", 1e6),
                ("in2", 0.000_645_16),
                ("ft2", 0.092_903_04),
                ("acre", 4_046.856_422_4),
            ],
            Category::Volume => &[
                ("ml", 0.001),
                ("l", 1.0),
                ("m3", 1000.0),
                ("cm3", 0.001),
                ("in3", 0.016_387_064),
                ("ft3", 28.316_846_592),
                ("gal", 3.785_411_784),
                ("pt", 0.473_176_473),
            ],
            Category::Time => &[
                ("ns", 1e-9),
                ("us", 1e-6),
                ("ms", 0.001),
                ("s", 1.0),
                ("min", 60.0),
                ("h", 3600.0),
                ("d", 86_400.0),
                ("wk", 604_800.0),
            ],
            Category::Speed => &[
                ("m/s", 1.0),
                ("km/h", 0.277_777_777_777_777_8),
                ("mph", 0.447_04),
                ("kn", 0.514_444_444_444_444_5),
                ("ft/s", 0.3048),
            ],
            // Expressed in bits, so the SI and IEC prefixes are distinguishable:
            // a kB is 8e3 bits and a Kib is 1024 bits.
            Category::Data => &[
                ("b", 1.0),
                ("B", 8.0),
                ("kb", 1e3),
                ("kB", 8e3),
                ("Kib", 1024.0),
                ("KiB", 8192.0),
                ("Mib", 1_048_576.0),
                ("MiB", 8_388_608.0),
                ("Gib", 1_073_741_824.0),
                ("GiB", 8_589_934_592.0),
                ("Tib", 1_099_511_627_776.0),
                ("TiB", 8_796_093_022_208.0),
            ],
        }
    }
}

/// Resolve a unit name within a category.
pub fn factor(category: Category, unit: &str) -> Option<f64> {
    category
        .units()
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(unit))
        .map(|(_, factor)| *factor)
}

/// Convert between two units of the same category.
///
/// Temperature is handled separately because its scales have offsets, which a
/// single factor cannot express.
pub fn convert(category: Category, value: f64, from: &str, to: &str) -> Option<f64> {
    if category == Category::Temperature {
        return convert_temperature(value, from, to);
    }
    let from_factor = factor(category, from)?;
    let to_factor = factor(category, to)?;
    if to_factor == 0.0 {
        return None;
    }
    Some(value * from_factor / to_factor)
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Option<f64> {
    // Normalise to kelvin, then out to the target.
    let kelvin = if from.eq_ignore_ascii_case("c") {
        value + 273.15
    } else if from.eq_ignore_ascii_case("f") {
        (value - 32.0) * 5.0 / 9.0 + 273.15
    } else if from.eq_ignore_ascii_case("k") {
        value
    } else {
        return None;
    };

    Some(if to.eq_ignore_ascii_case("c") {
        kelvin - 273.15
    } else if to.eq_ignore_ascii_case("f") {
        (kelvin - 273.15) * 9.0 / 5.0 + 32.0
    } else if to.eq_ignore_ascii_case("k") {
        kelvin
    } else {
        return None;
    })
}

/// A parsed conversion request, such as `5 km to miles`.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub value: f64,
    pub from: String,
    pub to: String,
}

/// Parse `5 km to miles`, or `5km in miles`.
///
/// The unit names are matched case insensitively across every category, so
/// `5 km to mi` and `5 kg to lb` both work without the caller naming a category.
pub fn parse_request(text: &str) -> Option<Request> {
    let cleaned = text.trim();
    let lowered = cleaned.to_ascii_lowercase();

    let separator = [" to ", " in ", " -> ", "=>"]
        .iter()
        .filter_map(|marker| lowered.find(marker).map(|index| (index, marker.len())))
        .min_by_key(|(index, _)| *index);

    let (index, marker_len) = separator?;
    let left = cleaned[..index].trim();
    let right = cleaned[index + marker_len..].trim();

    let split_at = left
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e'))
        .unwrap_or(left.len());
    let (number, unit) = left.split_at(split_at);
    let from = unit.trim();
    if from.is_empty() || right.is_empty() {
        return None;
    }
    let value: f64 = number.trim().parse().ok()?;
    if !value.is_finite() {
        return None;
    }

    Some(Request {
        value,
        from: from.to_string(),
        to: right.to_string(),
    })
}

/// Run a parsed request, guessing the category from the two unit names.
pub fn run(request: &Request) -> Option<(Category, f64)> {
    for category in [
        Category::Length,
        Category::Mass,
        Category::Temperature,
        Category::Area,
        Category::Volume,
        Category::Time,
        Category::Speed,
        Category::Data,
    ] {
        if factor(category, &request.from).is_some() && factor(category, &request.to).is_some() {
            let result = convert(category, request.value, &request.from, &request.to)?;
            return Some((category, result));
        }
    }
    None
}

/// Evaluate a request written as text, e.g. `5 km to miles`.
pub fn evaluate(text: &str) -> Result<(Category, f64), String> {
    let request = parse_request(text).ok_or_else(|| format!("cannot read {text:?}"))?;
    run(&request).ok_or_else(|| {
        format!(
            "no unit pair matches {:?} and {:?}",
            request.from, request.to
        )
    })
}

/// Format a converted value for a display, trimming noise.
pub fn format_result(value: f64) -> String {
    if !value.is_finite() {
        return "Error".to_string();
    }
    let magnitude = value.abs();
    if magnitude != 0.0 && (magnitude < 1e-6 || magnitude >= 1e12) {
        return format!("{value:.6e}");
    }
    let mut text = format!("{value:.10}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}
