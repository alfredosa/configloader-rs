use configloader::ConfigLoader;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct IntegerConfig {
    i8_value: i8,
    i16_value: i16,
    i32_value: i32,
    i64_value: i64,
    i128_value: i128,
    isize_value: isize,
    u8_value: u8,
    u16_value: u16,
    u32_value: u32,
    u64_value: u64,
    u128_value: u128,
    usize_value: usize,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct ScalarConfig {
    bool_value: bool,
    char_value: char,
    string_value: String,
    f32_value: f32,
    f64_value: f64,
}

#[test]
fn loads_all_signed_and_unsigned_integer_types() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("I8_VALUE", "-8");
        std::env::set_var("I16_VALUE", "-1600");
        std::env::set_var("I32_VALUE", "-320000");
        std::env::set_var("I64_VALUE", "-6400000000");
        std::env::set_var("I128_VALUE", "-128000000000000000000");
        std::env::set_var("ISIZE_VALUE", "-42");
        std::env::set_var("U8_VALUE", "8");
        std::env::set_var("U16_VALUE", "1600");
        std::env::set_var("U32_VALUE", "320000");
        std::env::set_var("U64_VALUE", "6400000000");
        std::env::set_var("U128_VALUE", "128000000000000000000");
        std::env::set_var("USIZE_VALUE", "42");
    }

    let config = IntegerConfig::load().unwrap();

    assert_eq!(config.i8_value, -8);
    assert_eq!(config.i16_value, -1600);
    assert_eq!(config.i32_value, -320000);
    assert_eq!(config.i64_value, -6400000000);
    assert_eq!(config.i128_value, -128000000000000000000);
    assert_eq!(config.isize_value, -42);
    assert_eq!(config.u8_value, 8);
    assert_eq!(config.u16_value, 1600);
    assert_eq!(config.u32_value, 320000);
    assert_eq!(config.u64_value, 6400000000);
    assert_eq!(config.u128_value, 128000000000000000000);
    assert_eq!(config.usize_value, 42);
}

#[test]
fn loads_bool_char_string_and_float_types() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("BOOL_VALUE", "true");
        std::env::set_var("CHAR_VALUE", "x");
        std::env::set_var("STRING_VALUE", "hello");
        std::env::set_var("F32_VALUE", "3.5");
        std::env::set_var("F64_VALUE", "2.25");
    }

    let config = ScalarConfig::load().unwrap();

    assert!(config.bool_value);
    assert_eq!(config.char_value, 'x');
    assert_eq!(config.string_value, "hello");
    assert!((config.f32_value - 3.5).abs() < f32::EPSILON);
    assert!((config.f64_value - 2.25).abs() < f64::EPSILON);
}
