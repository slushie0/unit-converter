use num_enum::TryFromPrimitive;
use std::env::{self, args};
use std::io::{self, Write};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone)]
struct Unit {
    name: &'static str,
    symbol: &'static str,
    cli_key: Option<&'static str>, // symbol in ascii format for CLI selection
    // conversion factor relative to a base unit
    // e.g. for Mass, base = kg: pound = 0.453592
    to_base: f64,
    offset: f64,
}
impl Unit {
    const fn new(
        name: &'static str,
        symbol: &'static str,
        cli_key: Option<&'static str>,
        to_base: f64,
        offset: Option<f64>,
    ) -> Unit {
        Unit {
            name,
            symbol,
            cli_key,
            to_base,
            offset: match offset {
                Some(o) => o,
                None => 0.,
            },
        }
    }
}

const DISTANCE_UNITS: &[Unit] = &[
    Unit::new("Metre", "m", None, 1.0, None),
    Unit::new("Kilometre", "km", None, 1000.0, None),
    Unit::new("Centimetre", "cm", None, 0.01, None),
    Unit::new("Millimetre", "mm", None, 0.001, None),
    Unit::new("Mile", "mi", None, 1609.344, None),
    Unit::new("Yard", "yd", None, 0.9144, None),
    Unit::new("Foot", "ft", None, 0.3048, None),
    Unit::new("Inch", "in", None, 0.0254, None),
];

const VOLUME_UNITS: &[Unit] = &[
    Unit::new("Litre", "L", Some("l"), 1.0, None),
    Unit::new("Millilitre", "mL", Some("ml"), 0.001, None),
    Unit::new("Cubic Metre", "m³", Some("m3"), 1000.0, None),
    Unit::new("Gallon (US)", "gal", None, 3.78541, None),
    Unit::new("Quart (US)", "qt", None, 0.946353, None),
    Unit::new("Pint (US)", "pt", None, 0.473176, None),
    Unit::new("Cup (US)", "cup", None, 0.24, None),
    Unit::new("Fluid Ounce", "fl oz", None, 0.0295735, None),
];

const MASS_UNITS: &[Unit] = &[
    Unit::new("Kilogram", "kg", None, 1.0, None),
    Unit::new("Gram", "g", None, 0.001, None),
    Unit::new("Milligram", "mg", None, 0.000001, None),
    Unit::new("Tonne", "t", None, 1000.0, None),
    Unit::new("Pound", "lb", None, 0.453592, None),
    Unit::new("Ounce", "oz", None, 0.0283495, None),
    Unit::new("Stone", "st", None, 6.35029, None),
];

const TIME_UNITS: &[Unit] = &[
    Unit::new("Second", "s", None, 1.0, None),
    Unit::new("Millisecond", "ms", None, 0.001, None),
    Unit::new("Minute", "min", None, 60.0, None),
    Unit::new("Hour", "h", None, 3600.0, None),
    Unit::new("Day", "d", None, 86400.0, None),
    Unit::new("Week", "wk", None, 604800.0, None),
    Unit::new("Year", "yr", None, 31536000.0, None),
];

const TEMPERATURE_UNITS: &[Unit] = &[
    Unit::new("Kelvin", "K", Some("k"), 1.0, None),
    Unit::new("Celsius", "°C", Some("c"), 1.0, Some(273.15)),
    Unit::new("Fahrenheit", "°F", Some("f"), 5.0 / 9.0, Some(459.67)),
];

#[derive(Debug, TryFromPrimitive, EnumIter)]
#[repr(u32)]
enum Property {
    Distance = 1,
    Volume,
    Mass,
    Time,
    Temperature,
}
impl Property {
    fn units(&self) -> &'static [Unit] {
        match self {
            Property::Distance => DISTANCE_UNITS,
            Property::Volume => VOLUME_UNITS,
            Property::Mass => MASS_UNITS,
            Property::Time => TIME_UNITS,
            Property::Temperature => TEMPERATURE_UNITS,
        }
    }
}

enum Direction {
    From,
    To,
}

fn ask_property() -> Property {
    println!("What would you like to convert?");
    for (i, property) in Property::iter().enumerate() {
        println!("{} -> {:?}", i + 1, property);
    }
    ask(|input| {
        let n: u32 = input.parse().ok()?;
        Property::try_from(n).ok()
    })
}

fn ask_unit(property: &Property, direction: Direction) -> Unit {
    println!(
        "What unit would you like to convert {}?",
        match direction {
            Direction::From => "from",
            Direction::To => "to",
        }
    );
    for unit in property.units() {
        match unit.cli_key {
            Some(i) => println!("{} -> {} ({})", i, unit.name, unit.symbol),
            None => println!("{} -> {}", unit.symbol, unit.name),
        }
    }
    ask(|input| {
        property
            .units()
            .iter()
            .find(|u| u.cli_key.unwrap_or(u.symbol).to_lowercase() == input)
            .cloned()
    })
}

fn ask_quantity(unit: &Unit) -> f64 {
    println!("How many {}?", unit.symbol);
    ask(|input| input.parse().ok())
}

fn ask<T, F>(prompt: F) -> T
where
    F: Fn(&str) -> Option<T>,
{
    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim().to_lowercase();
        println!();
        match prompt(&input) {
            Some(value) => return value,
            None => println!("Invalid input, try again"),
        }
    }
}

fn format_result(n: f64) -> String {
    let s = format!("{:.10}", n);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn convert(quantity: f64, from_unit: Unit, to_unit: Unit) {
    let base = (quantity + from_unit.offset) * from_unit.to_base;
    let result = base / to_unit.to_base - to_unit.offset;

    println!(
        "{quantity} {} ({}) is equal to {} {} ({})",
        from_unit.symbol,
        from_unit.name,
        format_result(result),
        to_unit.symbol,
        to_unit.name,
    );
}

fn bad_argument() {
    println!(
        "\"cvrt {:?}\" is not a known function. Try \"cvrt help\" for more information",
        env::args()
    );
}

fn main() {
    //let args : Vec<String> = env::args().collect();
    let mut arguments = vec![];
    for (i, argument) in env::args().enumerate() {
        if i == 0 {
            continue;
        }
        arguments.push(argument);
    }

    if arguments.len() == 0 {
        let property: Property = ask_property();
        let from_unit: Unit = ask_unit(&property, Direction::From);
        let to_unit: Unit = ask_unit(&property, Direction::To);
        let quantity = ask_quantity(&from_unit);

        convert(quantity, from_unit, to_unit);
        return;
    }

    if arguments[0] == "help" {
        println!();
        println!("cvrt is a simple CLI unit converter utility");
        println!();
        println!("Usage:   cvrt [unit in]>[unit out] [quantity]");
        println!();
        println!("Example: cvrt kg>lb 95");
        println!("\n or try cvrt without any arguments\n for a step-by-step conversion");
        println!();
        println!("See \"cvrt units\" for a full list of what cvrt can do");
        return;
    }

    if arguments.len() != 2 {
        bad_argument();
    }

    let units: &Vec<&str> = &arguments[0].split(">").collect();
    let quantity: f64 = arguments[1].parse().ok().expect("Quantity should be a number");

    let mut from_unit: Option<&Unit> = None;
    let mut to_unit: Option<&Unit> = None;

    for property in Property::iter() {
        let from = property
            .units()
            .iter()
            .find(|u| u.cli_key.unwrap_or(u.symbol).to_lowercase() == units[0]);
        if from.is_some() {
            from_unit = from;
            break;
        }
    }
    for property in Property::iter() {
        let to = property
            .units()
            .iter()
            .find(|u| u.cli_key.unwrap_or(u.symbol).to_lowercase() == units[1]);
        if to.is_some() {
            to_unit = to;
            break;
        }
    }
    convert(
        quantity,
        from_unit.expect("Invalid conversion from unit").clone(),
        to_unit.expect("Invalid conversion to unit").clone()
    );
}
