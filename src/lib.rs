use regex::Regex;
use std::{collections::HashSet, fs};

use serde::Deserialize;

#[derive(PartialEq, Debug)]
pub enum BairiakError {
    ReadSpecError,
    DeserializeYamlError,
    ParseBairiakEnumsError,
    WriteFileError,
    PositionOutOfRangeError,
}

#[derive(Debug)]
pub enum Bairiak {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

pub trait BairiakEnum {
    fn get_zero_bairiak() -> Bairiak;
    fn to_u8(self) -> u8;
}

impl Bairiak {
    pub fn is_false<B: BairiakEnum>(&self, flag: B) -> bool {
        match self {
            Bairiak::U8(value) => *value & 1u8 << flag.to_u8() == 0,
            Bairiak::U16(value) => *value & 1u16 << flag.to_u8() == 0,
            Bairiak::U32(value) => *value & 1u32 << flag.to_u8() == 0,
            Bairiak::U64(value) => *value & 1u64 << flag.to_u8() == 0,
            Bairiak::U128(value) => *value & 1u128 << flag.to_u8() == 0,
        }
    }

    pub fn is_true<B: BairiakEnum>(&self, flag: B) -> bool {
        !self.is_false(flag)
    }
}

#[derive(Debug, Deserialize)]
struct EnumSpec {
    enums: Vec<Enum>,
}

#[derive(Debug, Deserialize)]
struct Enum {
    name: String,
    variants: Vec<String>,
}

fn is_camel_case(s: &str) -> bool {
    let re = Regex::new(r"^[^a-z0-9][\w]*$").unwrap();
    re.is_match(s)
}

fn generete_zero_bairiak(variants_len: usize) -> Result<String, BairiakError> {
    let zero_bairiak = match variants_len {
        0..8 => "Bairiak::U8(0u8)",
        8..16 => "Bairiak::U16(0u16)",
        16..32 => "Bairiak::U32(0u32)",
        32..64 => "Bairiak::U64(0u64)",
        64..128 => "Bairiak::U128(0u128)",
        err => {
            eprintln!(
                "Error parsing Bairiak enums\nError: Position out of range: {}. Maximum positions supported is 128.",
                err
            );
            return Err(BairiakError::PositionOutOfRangeError);
        }
    };

    Ok(zero_bairiak.to_string())
}

fn validate_enum(name: &str, variants: &Vec<String>) -> Result<(), BairiakError> {
    if !is_camel_case(name) {
        eprintln!("Error parsing Bairiak enums\nError: Invalid enum name. Enum name should be in CamelCase.");
        return Err(BairiakError::ParseBairiakEnumsError);
    }

    if variants.is_empty() {
        eprintln!("Error parsing Bairiak enums\nError: Enum variants cannot be empty.");
        return Err(BairiakError::ParseBairiakEnumsError);
    }

    return Ok(());
}

fn generate_enum(e: &Enum) -> Result<String, BairiakError> {
    validate_enum(&e.name, &e.variants)?;

    let mut enum_code = format!(
        "
#[repr(u8)]
#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Debug)]
enum {} {{
",
        e.name
    );

    let zero_bairiak = generete_zero_bairiak(e.variants.len())?;

    for i in 0..e.variants.len() {
        let Some(v) = e.variants.get(i) else {
            eprintln!("Error parsing Bairiak enums");
            return Err(BairiakError::ParseBairiakEnumsError);
        };

        if !is_camel_case(v) {
            eprintln!("Error parsing Bairiak enums\nError: Invalid enum variant. Enum variant should be in CamelCase.");
            return Err(BairiakError::ParseBairiakEnumsError);
        }

        let variant = &format!("    {} = {},\n", v, i);
        enum_code.push_str(variant);
    }

    enum_code.push_str(&format!(
        "}}

impl BairiakEnum for {} {{
    fn get_zero_bairiak() -> Bairiak {{
        {}
    }}

    fn to_u8(self) -> u8 {{
        self as u8
    }}
}}\n",
        e.name, zero_bairiak,
    ));

    Ok(enum_code)
}

fn generate_enums(enums: &EnumSpec) -> Result<String, BairiakError> {
    let mut enums_code = String::new();
    for e in &enums.enums {
        enums_code.push_str(&generate_enum(e)?);
    }
    Ok(enums_code)
}

pub fn generate_bairiak_enums(
    bairiak_spec_path: &str,
    output_path: &str,
) -> Result<(), BairiakError> {
    let yaml_content = match fs::read_to_string(bairiak_spec_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            return Err(BairiakError::ReadSpecError);
        }
    };

    let enums = match serde_yaml::from_str(&yaml_content) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            return Err(BairiakError::DeserializeYamlError);
        }
    };

    let imports_code = "use bairiak::{Bairiak, BairiakEnum};";

    let enums_code = generate_enums(&enums)?;

    let bairiak_enums_code = format!("{}\n{}", imports_code, enums_code);

    match fs::write(output_path, bairiak_enums_code) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error writing file: {}", err);
            return Err(BairiakError::WriteFileError);
        }
    };

    Ok(())
}

pub fn generate_bairiak<B: BairiakEnum>(flags: HashSet<B>) -> Bairiak {
    let mut bairiak = B::get_zero_bairiak();
    for flag in flags {
        let flag_value = flag.to_u8();
        match &mut bairiak {
            Bairiak::U8(ref mut value) => *value |= 1u8 << flag_value,
            Bairiak::U16(ref mut value) => *value |= 1u16 << flag_value,
            Bairiak::U32(ref mut value) => *value |= 1u32 << flag_value,
            Bairiak::U64(ref mut value) => *value |= 1u64 << flag_value,
            Bairiak::U128(ref mut value) => *value |= 1u128 << flag_value,
        }
    }
    bairiak
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, sync::Once};

    static TEST_TEARDOWN: Once = Once::new();

    // Test for Bairiak::is_false and Bairiak::is_true methods
    #[repr(u8)]
    #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
    enum TestEnum {
        Flag0 = 0,
        Flag1 = 1,
        Flag2 = 2,
    }

    impl BairiakEnum for TestEnum {
        fn get_zero_bairiak() -> Bairiak {
            Bairiak::U8(0u8)
        }

        fn to_u8(self) -> u8 {
            self as u8
        }
    }

    #[test]
    fn test_bairiak_is_false() {
        let bairiak = Bairiak::U8(0);
        assert!(bairiak.is_false(TestEnum::Flag0));
        assert!(bairiak.is_false(TestEnum::Flag1));
        assert!(bairiak.is_false(TestEnum::Flag2));
    }

    #[test]
    fn test_bairiak_is_true() {
        let bairiak = Bairiak::U8(0b101);
        assert!(bairiak.is_true(TestEnum::Flag0));
        assert!(bairiak.is_false(TestEnum::Flag1));
        assert!(bairiak.is_true(TestEnum::Flag2));
    }

    // Test for generating Bairiak from a set of flags
    #[test]
    fn test_generate_bairiak() {
        let mut flags = HashSet::new();
        flags.insert(TestEnum::Flag0);
        flags.insert(TestEnum::Flag2);

        let bairiak = generate_bairiak(flags);
        match bairiak {
            Bairiak::U8(value) => assert_eq!(value, 0b101),
            _ => panic!("Expected Bairiak::U8"),
        }
    }

    // Test for generating enums from Enum struct with invalid name
    #[test]
    fn test_generate_enum_with_invalid_name() {
        let e = Enum {
            name: String::from("1"),
            variants: vec![
                String::from("Var0"),
                String::from("Var1"),
                String::from("Var2"),
            ],
        };

        let result = generate_enum(&e);
        assert!(matches!(result, Err(BairiakError::ParseBairiakEnumsError)));
    }

    #[test]
    fn test_generate_enum_with_invalid_variant_lowercase() {
        let e = Enum {
            name: String::from("TestEnum"),
            variants: vec![
                String::from("Var0"),
                String::from("var1"),
                String::from("Var2"),
            ],
        };

        let result = generate_enum(&e);
        assert!(matches!(result, Err(BairiakError::ParseBairiakEnumsError)));
    }

    #[test]
    fn test_generate_enum_with_invalid_variant_number() {
        let e = Enum {
            name: String::from("TestEnum"),
            variants: vec![
                String::from("Var0"),
                String::from("1var"),
                String::from("Var2"),
            ],
        };

        let result = generate_enum(&e);
        assert!(matches!(result, Err(BairiakError::ParseBairiakEnumsError)));
    }

    #[test]
    fn test_generate_enum_with_invalid_variant_symbol() {
        let e = Enum {
            name: String::from("TestEnum"),
            variants: vec![
                String::from("Var0"),
                String::from("var!"),
                String::from("Var2"),
            ],
        };

        let result = generate_enum(&e);
        assert!(matches!(result, Err(BairiakError::ParseBairiakEnumsError)));
    }

    // Test for generating enums from Enum struct with empty variants
    #[test]
    fn test_generate_enum_with_empty_variants() {
        let e = Enum {
            name: String::from("TestEnum"),
            variants: vec![],
        };

        let result = generate_enum(&e);
        assert!(matches!(result, Err(BairiakError::ParseBairiakEnumsError)));
    }

    // Test for generating enums from Enum struct
    #[test]
    fn test_generate_enum() {
        let e = Enum {
            name: String::from("TestEnum"),
            variants: vec![
                String::from("Var0"),
                String::from("Var1"),
                String::from("Var2"),
            ],
        };

        let result = generate_enum(&e);
        assert!(result.is_ok());

        let generated_code = result.unwrap();
        assert!(generated_code.contains("enum TestEnum"));
        assert!(generated_code.contains("Var0 = 0"));
        assert!(generated_code.contains("Var1 = 1"));
        assert!(generated_code.contains("Var2 = 2"));
    }

    // Test for the overall enum generation function
    #[test]
    fn test_generate_enums() {
        let spec = EnumSpec {
            enums: vec![Enum {
                name: String::from("TestEnum"),
                variants: vec![String::from("Var0"), String::from("Var1")],
            }],
        };

        let result = generate_enums(&spec);
        assert!(result.is_ok());

        let generated_code = result.unwrap();
        assert!(generated_code.contains("enum TestEnum"));
        assert!(generated_code.contains("Var0 = 0"));
        assert!(generated_code.contains("Var1 = 1"));
    }

    // Test for file generation success case
    #[test]
    fn test_generate_bairiak_enums_success() {
        let result = generate_bairiak_enums("test_data/valid_spec.yaml", "output.rs");
        assert!(result.is_ok());
        TEST_TEARDOWN.call_once(|| {
            fs::remove_file("output.rs").unwrap();
        });
    }

    #[test]
    fn test_generate_bairiak_enums_with_more_than_max_flags() {
        let result = generate_bairiak_enums("test_data/out_of_range_spec.yaml", "output.rs");
        assert!(matches!(result, Err(BairiakError::PositionOutOfRangeError)));
    }

    // Test for file generation failure due to missing file
    #[test]
    fn test_generate_bairiak_enums_file_not_found() {
        let result = generate_bairiak_enums("non_existent_file.yaml", "output.rs");
        assert!(matches!(result, Err(BairiakError::ReadSpecError)));
    }

    // Test for file generation failure due to invalid YAML
    #[test]
    fn test_generate_bairiak_enums_invalid_yaml() {
        let result = generate_bairiak_enums("test_data/invalid_spec.yaml", "output.rs");
        assert!(matches!(result, Err(BairiakError::DeserializeYamlError)));
    }
}
