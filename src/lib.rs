use chrono::{Datelike, NaiveDate, Utc};
use phf::phf_ordered_map;
use regex::Regex;
use std::{error::Error, fmt};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

/// Check if the string provided is a valid Italian Fiscal Code.
/// Temporary codes are supported.
pub fn validate(code: &str) -> bool {
    validate_or_error(code).is_ok()
}

/// Check if the string provided is a valid Italian Fiscal Code.
/// Temporary codes are supported.
pub fn validate_or_error(code: &str) -> Result<(), Box<dyn Error>> {
    let code = code.trim().to_uppercase();
    let regex = Regex::new(r"^\d{11}$").expect("valid regex");
    if regex.is_match(&code) {
        // temporary fiscal code
        let (code, check_character) = code.split_at(10);
        return if check_character == calculate_check_character_temporary(code).to_string() {
            Ok(())
        } else {
            Err("Invalid temporary fiscal code".into())
        };
    }

    FiscalCode::try_from(code.as_str()).map(|_| ())
}

/// This function expects a valid Italian Fiscal Code as input.
///
/// You can use [validate] to check if the code is correct before calling this.
/// Note that temporary codes are **not** supported.
pub fn info(code: &str) -> Result<FiscalCodeInfo, Box<dyn Error>> {
    let code = FiscalCode::try_from(code)?;

    Ok(FiscalCodeInfo {
        born_on: code.born_on,
        gender: code.gender,
        place_of_birth: code.place_of_birth,
    })
}

#[derive(Debug, Clone)]
pub struct FiscalCodeInfo {
    pub born_on: NaiveDate,
    pub gender: Gender,
    pub place_of_birth: PlaceOfBirth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Gender {
    Female,
    Male,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Gender::Female => "F",
                Gender::Male => "M",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct PlaceOfBirth {
    pub country_code: String,
    pub country_name: String,
    pub city: Option<String>,
    pub state: Option<String>,
}

impl fmt::Display for PlaceOfBirth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Country: {} ({})\n\tCity: {} ({})",
            self.country_name,
            self.country_code,
            self.city.clone().unwrap_or("N/A".into()),
            self.state.clone().unwrap_or("N/A".into())
        )
    }
}

fn calculate_check_character(code: &str) -> char {
    let mut sum = 0;
    for (i, character) in code[..code.len() - 1].char_indices() {
        if (i + 1) % 2 == 0 {
            sum += CHECK_CHARACTER_EVEN_REPLACEMENTS
                .get(&character)
                .expect("character replacement found");
        } else {
            sum += CHECK_CHARACTER_ODD_REPLACEMENTS
                .get(&character)
                .expect("character replacement found");
        }
    }

    CHECK_CHARACTER_REMINDER
        .get(&(sum % 26))
        .copied()
        .expect("value replacement found")
}

fn calculate_check_character_temporary(code: &str) -> char {
    let digits: Vec<u8> = code
        .chars()
        .map(|c| c.to_digit(10).expect("valid digit") as u8)
        .collect();

    let odd_sum: u8 = digits.iter().step_by(2).sum();
    let even_sum: u8 = digits
        .iter()
        .skip(1)
        .step_by(2)
        .map(|&digit| {
            let doubled = digit * 2;
            if doubled >= 10 {
                doubled - 9
            } else {
                doubled
            }
        })
        .sum();

    let total = odd_sum + even_sum;
    let units = total % 10;
    ((10 - units) % 10 + 48) as char
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct FiscalCode {
    /// The string representing this code
    representation: String,
    /// The string representing this code without any omocodia alterations
    representation_canonical: String,
    surname: String,
    name: String,
    born_on: NaiveDate,
    gender: Gender,
    place_of_birth: PlaceOfBirth,
}

impl TryFrom<&str> for FiscalCode {
    type Error = Box<dyn Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let code = s.trim().to_uppercase();
        if code.len() != 16 {
            return Err("Invalid length".into());
        }
        let regex = Regex::new(r"([A-Z]{3})([A-Z]{3})(\d{2})([A-Z])(\d{2})([A-Z]\d{3})([A-Z])")
            .expect("valid regex");

        let check_character_calculated = calculate_check_character(&code.to_string());

        // get the original code that may be modified in case of omocodia
        let code_canonical: String = {
            let indices = [6usize, 7, 9, 10, 12, 13, 14];
            code.char_indices()
                .map(|(i, character)| {
                    if indices.contains(&i) {
                        DIGIT_REPLACEMENTS
                            .into_iter()
                            .find(|(_, &value)| value == character)
                            // convert to the correct ASCII char
                            .map_or(character, |(&key, _)| (key + 48) as char)
                    } else {
                        character
                    }
                })
                .collect()
        };

        if let Some(captures) = regex.captures(&code_canonical) {
            let birth_year = captures.get(3).unwrap().as_str().parse().unwrap();
            let birth_month = captures.get(4).unwrap().as_str().chars().next().unwrap();
            let birth_day_gender = captures.get(5).unwrap().as_str().parse().unwrap();
            let birth_town = captures.get(6).unwrap().as_str();
            let check_character_actual = captures.get(7).unwrap().as_str().chars().next().unwrap();

            if check_character_actual != check_character_calculated {
                return Err(format!(
                    "Invalid check character: found {}, expected {}",
                    check_character_actual, check_character_calculated,
                )
                .into());
            }

            Ok(FiscalCode {
                representation: code,
                representation_canonical: captures.get(0).unwrap().as_str().into(),
                surname: captures.get(1).unwrap().as_str().into(),
                name: captures.get(2).unwrap().as_str().into(),
                born_on: born_on(birth_year, birth_month, birth_day_gender)?,
                gender: gender(birth_day_gender),
                place_of_birth: place_of_birth(birth_town)?,
            })
        } else {
            Err("Invalid fiscal code format".into())
        }
    }
}

fn born_on(
    birth_year: u8,
    birth_month: char,
    birth_day_gender: u8,
) -> Result<NaiveDate, Box<dyn Error>> {
    let day = if birth_day_gender > 40 {
        birth_day_gender - 40
    } else {
        birth_day_gender
    };

    let month = *BIRTH_MONTHS
        .entries()
        .find(|(_, &c)| c == birth_month)
        .ok_or("Invalid birth month")?
        .0
        + 1;

    let year = {
        let current = Utc::now().year() as f32;

        let year = ((current / 100.0).round() * 100.0) as i32 + birth_year as i32;

        if year < current as i32 {
            year
        } else {
            year - 100
        }
    };

    Ok(NaiveDate::from_ymd_opt(year, month.into(), day.into()).ok_or("Invalid birth date")?)
}

fn gender(birth_day_gender: u8) -> Gender {
    if birth_day_gender > 40 {
        Gender::Female
    } else {
        Gender::Male
    }
}

fn place_of_birth(birth_town: &str) -> Result<PlaceOfBirth, Box<dyn Error>> {
    let location = *BIRTH_TOWNS.get(birth_town).ok_or("Invalid birth town")?;

    Ok(PlaceOfBirth {
        country_code: location.country_code.into(),
        country_name: location.country_name.into(),
        city: location.city.map(|v| v.into()),
        state: location.state.map(|v| v.into()),
    })
}

static BIRTH_MONTHS: phf::OrderedMap<u8, char> = phf_ordered_map! {
    0u8 => 'A',
    1u8 => 'B',
    2u8 => 'C',
    3u8 => 'D',
    4u8 => 'E',
    5u8 => 'H',
    6u8 => 'L',
    7u8 => 'M',
    8u8 => 'P',
    9u8 => 'R',
    10u8 => 'S',
    11u8 => 'T',
};

static DIGIT_REPLACEMENTS: phf::OrderedMap<u8, char> = phf_ordered_map! {
   0u8 => 'L',
   1u8 => 'M',
   2u8 => 'N',
   3u8 => 'P',
   4u8 => 'Q',
   5u8 => 'R',
   6u8 => 'S',
   7u8 => 'T',
   8u8 => 'U',
   9u8 => 'V',
};

static CHECK_CHARACTER_ODD_REPLACEMENTS: phf::OrderedMap<char, u8> = phf_ordered_map! {
   '0' => 1u8,
   '1' => 0u8,
   '2' => 5u8,
   '3' => 7u8,
   '4' => 9u8,
   '5' => 13u8,
   '6' => 15u8,
   '7' => 17u8,
   '8' => 19u8,
   '9' => 21u8,
   'A' => 1u8,
   'B' => 0u8,
   'C' => 5u8,
   'D' => 7u8,
   'E' => 9u8,
   'F' => 13u8,
   'G' => 15u8,
   'H' => 17u8,
   'I' => 19u8,
   'J' => 21u8,
   'K' => 2u8,
   'L' => 4u8,
   'M' => 18u8,
   'N' => 20u8,
   'O' => 11u8,
   'P' => 3u8,
   'Q' => 6u8,
   'R' => 8u8,
   'S' => 12u8,
   'T' => 14u8,
   'U' => 16u8,
   'V' => 10u8,
   'W' => 22u8,
   'X' => 25u8,
   'Y' => 24u8,
   'Z' => 23u8,
};

static CHECK_CHARACTER_EVEN_REPLACEMENTS: phf::OrderedMap<char, u8> = phf_ordered_map! {
   '0' => 0u8,
   '1' => 1u8,
   '2' => 2u8,
   '3' => 3u8,
   '4' => 4u8,
   '5' => 5u8,
   '6' => 6u8,
   '7' => 7u8,
   '8' => 8u8,
   '9' => 9u8,
   'A' => 0u8,
   'B' => 1u8,
   'C' => 2u8,
   'D' => 3u8,
   'E' => 4u8,
   'F' => 5u8,
   'G' => 6u8,
   'H' => 7u8,
   'I' => 8u8,
   'J' => 9u8,
   'K' => 10u8,
   'L' => 11u8,
   'M' => 12u8,
   'N' => 13u8,
   'O' => 14u8,
   'P' => 15u8,
   'Q' => 16u8,
   'R' => 17u8,
   'S' => 18u8,
   'T' => 19u8,
   'U' => 20u8,
   'V' => 21u8,
   'W' => 22u8,
   'X' => 23u8,
   'Y' => 24u8,
   'Z' => 25u8,
};

static CHECK_CHARACTER_REMINDER: phf::OrderedMap<u8, char> = phf_ordered_map! {
   0u8 => 'A',
   1u8 => 'B',
   2u8 => 'C',
   3u8 => 'D',
   4u8 => 'E',
   5u8 => 'F',
   6u8 => 'G',
   7u8 => 'H',
   8u8 => 'I',
   9u8 => 'J',
   10u8 => 'K',
   11u8 => 'L',
   12u8 => 'M',
   13u8 => 'N',
   14u8 => 'O',
   15u8 => 'P',
   16u8 => 'Q',
   17u8 => 'R',
   18u8 => 'S',
   19u8 => 'T',
   20u8 => 'U',
   21u8 => 'V',
   22u8 => 'W',
   23u8 => 'X',
   24u8 => 'Y',
   25u8 => 'Z',
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        //spell-checker: disable
        assert!(validate("GNTMTT99C27H501F"));
        assert!(validate("MRARSS80A01H501T"));
        assert!(validate("BNCLRD69T61A783M"));
        assert!(validate("FCKTSS05C01Z122F"));
        assert!(validate("FCKTSS05C01ZMLQH"));

        assert!(!validate("INVALIDCODE"));
        assert!(!validate("FCKTSS05C01Z122K"));
        assert!(!validate("FCKTSS05F01Z122F"));
        assert!(!validate("FCKTSS05C32Z122F"));
        assert!(!validate("FCKTSS05C01Z105L"));
        assert!(!validate("GNTMTT99C72H501Y"));
        //spell-checker: enable
    }

    #[test]
    fn test_validate_omocodia() {
        //spell-checker: disable
        assert!(validate("GNTMTT99C27H50MX"));
        assert!(validate("GNTMTT99C27HR0MS"));
        //spell-checker: enable
    }

    #[test]
    fn test_validate_provisional() {
        assert!(validate("12345678903"));
    }

    #[test]
    fn test_validate_empty() {
        assert!(!validate(""));
    }

    #[test]
    fn test_validate_invalid_length() {
        // spell-checker: disable
        assert!(!validate("TOOSHORT"));
        assert!(!validate("THISCODEISTOOLONGTOBEAVALIDFISCALCODE"));
        //spell-checker: enable
    }

    #[test]
    fn test_info() {
        //spell-checker: disable
        let info = super::info("GNTMTT99C27H501F");
        //spell-checker: enable
        assert!(info.is_ok());
        assert_eq!(
            info.as_ref().unwrap().born_on,
            NaiveDate::from_ymd_opt(1999, 3, 27).unwrap()
        );
        assert_eq!(info.as_ref().unwrap().gender, Gender::Male);
        assert_eq!(info.as_ref().unwrap().place_of_birth.country_name, "Italia");
        assert_eq!(info.as_ref().unwrap().place_of_birth.country_code, "IT");
        assert_eq!(
            info.as_ref().unwrap().place_of_birth.city,
            Some("Roma".into()),
        );
        assert_eq!(
            info.as_ref().unwrap().place_of_birth.state,
            Some("RM".into()),
        );

        //spell-checker: disable
        let info = super::info("MKSKRS92L65Z219S");
        //spell-checker: enable
        assert!(info.is_ok());
        assert_eq!(
            info.as_ref().unwrap().born_on,
            NaiveDate::from_ymd_opt(1992, 7, 25).unwrap()
        );
        assert_eq!(info.as_ref().unwrap().gender, Gender::Female);
        assert_eq!(
            info.as_ref().unwrap().place_of_birth.country_name,
            "Giappone"
        );
        assert_eq!(info.as_ref().unwrap().place_of_birth.country_code, "JP");
        assert!(info.as_ref().unwrap().place_of_birth.city.is_none());
        assert!(info.as_ref().unwrap().place_of_birth.state.is_none());
    }
}
