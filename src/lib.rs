//! A parsing library for the [reaclib] data format.
//!
//! The data is represented by [`Set`], and the parsing is mostly done by [`Iter`].
//! The data can be collected into a type that implements [`FromIterator`], such as [`Vec`].
//! A convenience function [`to_hash_map`] is provided for the case that you want a `Vec` of all
//! `Set`s for each reaction.
//!
//! [reaclib]: https://reaclib.jinaweb.org/
//!
//! # Format
//!
//! The format is documented on the [reaclib format help page][reaclib_format].
//! There are two formats, both supported by this library.
//! [`Format`] is used to indicate which version to expect.
//!
//! [reaclib_format]: https://reaclib.jinaweb.org/help.php?topic=reaclib_format
//!
//! # Examples
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use reaclib::{Format, Iter, Set};
//! use std::{fs::File, io::BufReader};
//!
//! let file = File::open("reaclib")?;
//! let file = BufReader::new(file);
//! let iter = Iter::new(file, Format::Reaclib1);
//! let data: Vec<Set> = iter.collect::<Result<_, _>>()?;
//! # Ok(())
//! # }
//! ```
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use reaclib::{Format, Reaction, Set, to_hash_map};
//! use std::{collections::HashMap, io::stdin};
//!
//! let input = stdin().lock();
//! let data: HashMap<Reaction, Vec<Set>> = to_hash_map(input, Format::Reaclib2)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! * `serde`: Provide `Serialize` and `Deserialize` implementations for [serde](https://serde.rs).
use crate::error::ReaclibError as RError;
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Unstructured};
use arrayvec::{ArrayString, ArrayVec};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{BufRead, Lines},
    ops::Range,
    str::FromStr,
};

pub use crate::error::ReaclibError;

mod error;
#[cfg(test)]
mod tests;

/// A type that represents a nuclide.
pub type Nuclide = ArrayString<5>;

/// A type that represents a reaction.
///
/// The first element represents the reactants and the second element represents the products.
pub type Reaction = (ArrayVec<Nuclide, 4>, ArrayVec<Nuclide, 4>);

/// A type holding a single set of reaclib data.
///
/// A reaction may be made up of multiple sets.
///
/// ```
/// use reaclib::{Format, Iter};
/// use std::io::Cursor;
///
/// let reader = Cursor::new(r"1
///          n    p                            wc12w     7.82300e-01          
/// -6.781610e+00 0.000000e+00 0.000000e+00 0.000000e+00                      
///  0.000000e+00 0.000000e+00 0.000000e+00                                   ");
///
/// let mut iter = Iter::new(reader, Format::Reaclib2);
/// let data = iter.next().unwrap().unwrap();
/// assert_eq!(data.q_value, 7.82300e-01);
/// ```
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Set {
    /// The nuclides going into a reaction.
    pub reactants: ArrayVec<Nuclide, 4>,
    /// The nuclides resulting from a reaction.
    pub products: ArrayVec<Nuclide, 4>,
    /// A label denoting the source of the reaction.
    ///
    /// Here is a [list of all labels](https://reaclib.jinaweb.org/labels.php).
    pub label: ArrayString<4>,
    /// The resonance flag for the reaction.
    pub resonance: Resonance,
    /// A flag denoting whether the reaction rate was derived from the reverse rate using detailed
    /// balance.
    ///
    /// Rates with this flag set, must be corrected to include partition function modifications.
    pub reverse: bool,
    /// The Q-value of the reaction.
    pub q_value: f64,
    /// The parameters of this reaction rate set.
    ///
    /// See the [reaclib format help](https://reaclib.jinaweb.org/help.php?topic=reaclib_format)
    /// for how to interpret these parameters, and [`rate`][Self::rate] for an implementation of
    /// that.
    pub params: [f64; 7],
}

impl Set {
    fn from_lines(chapter: Chapter, lines: &[String; 3]) -> Result<Self, RError> {
        fn range_err(line: &str, range: Range<usize>) -> Result<&str, RError> {
            if line.len() < range.end {
                Err(RError::TooShortLine)
            } else {
                Ok(line.get(range).ok_or(RError::StrIndex)?.trim())
            }
        }

        let reactants = (0..chapter.num_reactants())
            .map(|i| {
                let r = (5 + 5 * i)..(5 + 5 * (i + 1));
                Ok(Nuclide::from(range_err(&lines[0], r)?)
                    .expect("the range is 5 and the capacity is 5"))
            })
            .collect::<Result<_, RError>>()?;
        let products = (chapter.num_reactants()
            ..(chapter.num_reactants() + chapter.num_products()))
            .map(|i| {
                let r = (5 + 5 * i)..(5 + 5 * (i + 1));
                Ok(Nuclide::from(range_err(&lines[0], r)?)
                    .expect("the range is 5 and the capacity is 5"))
            })
            .collect::<Result<_, RError>>()?;
        let label = ArrayString::from(range_err(&lines[0], 43..47)?)
            .expect("the range is 4 and the capacity is 4");
        let resonance = range_err(&lines[0], 47..48)?.parse()?;
        let reverse = range_err(&lines[0], 48..49)? == "v";
        let q_value = range_err(&lines[0], 52..64)?.parse()?;
        let params = [
            range_err(&lines[1], 0..13)?.parse()?,
            range_err(&lines[1], 13..26)?.parse()?,
            range_err(&lines[1], 26..39)?.parse()?,
            range_err(&lines[1], 39..52)?.parse()?,
            range_err(&lines[2], 0..13)?.parse()?,
            range_err(&lines[2], 13..26)?.parse()?,
            range_err(&lines[2], 26..39)?.parse()?,
        ];

        Ok(Self {
            reactants,
            products,
            label,
            resonance,
            reverse,
            q_value,
            params,
        })
    }

    /// Calculate the rate based on the rate parameters and their meaning, accoriding to the
    /// [reaclib format help](https://reaclib.jinaweb.org/help.php?topic=reaclib_format).
    #[must_use]
    pub fn rate(&self, temperature: f64) -> f64 {
        // the indexing here can panic if the index is out of bounds, but `params` has a len of 7,
        // so indices of 0..=6 will not cause a panic
        // also, be careful with `i as f64`. this is fine because 0..=6 can all be represented by f64
        #[allow(clippy::cast_precision_loss)]
        let sum = (1..=5)
            .map(|i| self.params[i] * f64::powf(temperature, 2.0 * (i as f64) * 5.0 / 3.0))
            .sum::<f64>();
        f64::exp(self.params[6].mul_add(f64::ln(temperature), self.params[0] + sum))
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for Set {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        // this is adapted from arbitrary's implementation of Arbitrary for &str
        fn array_string<const CAP: usize>(
            u: &mut Unstructured,
        ) -> arbitrary::Result<ArrayString<CAP>> {
            let size = usize::min(u.arbitrary_len::<u8>()?, CAP);
            match std::str::from_utf8(u.peek_bytes(size).unwrap()) {
                Ok(s) => {
                    u.bytes(size).unwrap();
                    Ok(ArrayString::from(s).expect("size is limited to CAP"))
                }
                Err(e) => {
                    let i = e.valid_up_to();
                    let valid = u.bytes(i).unwrap();
                    let s = ArrayString::from(
                        std::str::from_utf8(valid).expect("we already checked for validity"),
                    )
                    .expect("size is limited to CAP");
                    Ok(s)
                }
            }
        }

        let chapter: Chapter = u.arbitrary()?;

        let mut reactants = ArrayVec::new();
        for _ in 0..(chapter.num_reactants()) {
            reactants.push(array_string(u)?);
        }
        let mut products = ArrayVec::new();
        for _ in 0..(chapter.num_products()) {
            products.push(array_string(u)?);
        }
        let label = array_string(u)?;
        let resonance = u.arbitrary()?;
        let reverse = u.arbitrary()?;
        let q_value = u.arbitrary()?;
        let params = u.arbitrary()?;

        Ok(Self {
            reactants,
            products,
            label,
            resonance,
            reverse,
            q_value,
            params,
        })
    }
}

/// A flag denoting whether a reaction is resonant, non-resonant, or weak.
///
/// There is also an undocumented "s" variant.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[non_exhaustive]
pub enum Resonance {
    NonResonant,
    Resonant,
    Weak,
    S,
}

impl FromStr for Resonance {
    type Err = RError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" | " " | "n" => Ok(Self::NonResonant),
            "r" => Ok(Self::Resonant),
            "w" => Ok(Self::Weak),
            "s" => Ok(Self::S),
            _ => Err(RError::UnknownResonance(s.to_string())),
        }
    }
}

/// A type used to specify how a reaclib file should be parsed.
///
/// REACLIB 1 (R1) and REACLIB 2 (R2) are both supported by this library.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[non_exhaustive]
pub enum Format {
    /// A three-line chapter header precedes multiple set entries.
    Reaclib1,
    /// A one-line chapter header precedes each set entry.
    Reaclib2,
}

/// A type that describes a class of reactions with the same number of reactants and products.
///
/// Originally, Chapter 8 was used for both e1 + e2 + e3 → e4 and e1 + e2 + e3 → e4 + e5 reactions.
/// Chapter 8 now is only used for the first type, and Chapter 9 is used for the second type.
/// This library does not handle older reaclib files with both types in Chapter 8.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[non_exhaustive]
pub enum Chapter {
    /// e1 → e2
    Chapter1,
    /// e1 → e2 + e3
    Chapter2,
    /// e1 → e2 + e3 + e4
    Chapter3,
    /// e1 + e2 → e3
    Chapter4,
    /// e1 + e2 → e3 + e4
    Chapter5,
    /// e1 + e2 → e3 + e4 + e5
    Chapter6,
    /// e1 + e2 → e3 + e4 + e5 + e6
    Chapter7,
    /// e1 + e2 + e3 → e4
    Chapter8,
    /// e1 + e2 + e3 → e4 + e5
    Chapter9,
    /// e1 + e2 + e3 + e4 → e5 + e6
    Chapter10,
    /// e1 → e2 + e3 + e4 + e5
    Chapter11,
}

impl Chapter {
    #[must_use]
    pub const fn num_reactants(&self) -> usize {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Chapter1 => 1,
            Self::Chapter2 => 1,
            Self::Chapter3 => 1,
            Self::Chapter4 => 2,
            Self::Chapter5 => 2,
            Self::Chapter6 => 2,
            Self::Chapter7 => 2,
            Self::Chapter8 => 3,
            Self::Chapter9 => 3,
            Self::Chapter10 => 4,
            Self::Chapter11 => 1,
        }
    }

    #[must_use]
    pub const fn num_products(&self) -> usize {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Chapter1 => 1,
            Self::Chapter2 => 2,
            Self::Chapter3 => 3,
            Self::Chapter4 => 1,
            Self::Chapter5 => 2,
            Self::Chapter6 => 3,
            Self::Chapter7 => 4,
            Self::Chapter8 => 1,
            Self::Chapter9 => 2,
            Self::Chapter10 => 2,
            Self::Chapter11 => 4,
        }
    }

    // This may fail in two ways:
    //   * It is not a chapter header (`None`)
    //   * It is a chapter header, but parsing fails (`Some(Err(_))`)
    // This is because we try to parse a group of 3 lines as a chapter header first, and if that
    // fails, we try to parse it as a reaction set.
    fn from_lines_v1(lines: &[String; 3]) -> Option<Result<Self, RError>> {
        let [l1, l2, l3] = lines;

        if l2.trim().is_empty() && l3.trim().is_empty() {
            match l1.trim().parse::<u8>() {
                Ok(c) => Some(c.try_into()),
                Err(e) => Some(Err(e.into())),
            }
        } else {
            None
        }
    }

    // This is simpler than _v1 because a set *always* contains a (one-line) chapter header.
    // So if we can't parse it, that's an error.
    fn from_lines_v2(line: &str) -> Result<Self, RError> {
        line.trim().parse::<u8>()?.try_into()
    }
}

impl TryFrom<u8> for Chapter {
    type Error = RError;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            1 => Ok(Self::Chapter1),
            2 => Ok(Self::Chapter2),
            3 => Ok(Self::Chapter3),
            4 => Ok(Self::Chapter4),
            5 => Ok(Self::Chapter5),
            6 => Ok(Self::Chapter6),
            7 => Ok(Self::Chapter7),
            8 => Ok(Self::Chapter8),
            9 => Ok(Self::Chapter9),
            10 => Ok(Self::Chapter10),
            11 => Ok(Self::Chapter11),
            _ => Err(RError::UnknownChapter(x)),
        }
    }
}

/// An iterator that reads reaclib data.
///
/// # Examples
///
/// ```
/// use reaclib::{Iter, Format};
/// use std::io::Cursor;
///
/// // `Cursor` is a type that implements `BufRead`.
/// // Consider using `BufReader` if you have a `File`.
/// let data_v1 = Cursor::new(r"1                                                                         
///                                                                           
///                                                                           
///          n    p                            wc12w     7.82300e-01          
/// -6.781610e+00 0.000000e+00 0.000000e+00 0.000000e+00                      
///  0.000000e+00 0.000000e+00 0.000000e+00                                   ");
/// let mut iter = Iter::new(data_v1, Format::Reaclib1);
/// assert!(iter.next().is_some());
/// assert!(iter.next().is_none());
///
/// let data_v2 = Cursor::new(r"1
///          n    p                            wc12w     7.82300e-01          
/// -6.781610e+00 0.000000e+00 0.000000e+00 0.000000e+00                      
///  0.000000e+00 0.000000e+00 0.000000e+00                                   ");
/// let mut iter = Iter::new(data_v2, Format::Reaclib2);
/// assert!(iter.next().is_some());
/// assert!(iter.next().is_none());
/// ```
///
/// # Errors
///
/// If a set fails to parse or there is a reading error, [`next`][Self::next] will return `Some(Err)`.
/// Calling `next` again may return `Some`, but the validity of the data is not guaranteed.
pub struct Iter<R: BufRead> {
    lines: Lines<R>,
    format: Format,
    chapter: Option<Chapter>,
}

impl<R: BufRead> Iter<R> {
    /// Creates a new `Iter` from `reader`. It will be parsed according to the rules of `format`.
    pub fn new(reader: R, format: Format) -> Self {
        let lines = reader.lines();
        Self {
            lines,
            format,
            chapter: None,
        }
    }

    fn next_v1(&mut self) -> Option<<Self as Iterator>::Item> {
        loop {
            let lines = match (self.lines.next(), self.lines.next(), self.lines.next()) {
                (None, _, _) => return None,
                (_, None, _) | (_, _, None) => {
                    return Some(Err(RError::TooFewLines));
                }
                (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => {
                    return Some(Err(e.into()));
                }
                (Some(Ok(l1)), Some(Ok(l2)), Some(Ok(l3))) => [l1, l2, l3],
            };

            // Try to interpret as chapter header
            // if that fails, try to interpret as a set
            // it is an error to have a set if the chapter hasn't been set yet
            match Chapter::from_lines_v1(&lines) {
                Some(Ok(chapter)) => {
                    self.chapter = Some(chapter);
                    continue;
                }
                Some(Err(e)) => {
                    break Some(Err(e));
                }
                None => {
                    if let Some(chapter) = self.chapter {
                        break Some(Set::from_lines(chapter, &lines));
                    }
                    break Some(Err(RError::ChapterUnset));
                }
            }
        }
    }

    fn next_v2(&mut self) -> Option<<Self as Iterator>::Item> {
        let (ch_line, set_lines) = match (
            self.lines.next(),
            self.lines.next(),
            self.lines.next(),
            self.lines.next(),
        ) {
            (None, _, _, _) => return None,
            (_, None, _, _) | (_, _, None, _) | (_, _, _, None) => {
                return Some(Err(RError::TooFewLines));
            }
            (Some(Err(e)), _, _, _)
            | (_, Some(Err(e)), _, _)
            | (_, _, Some(Err(e)), _)
            | (_, _, _, Some(Err(e))) => {
                return Some(Err(e.into()));
            }
            (Some(Ok(l1)), Some(Ok(l2)), Some(Ok(l3)), Some(Ok(l4))) => (l1, [l2, l3, l4]),
        };

        match Chapter::from_lines_v2(&ch_line) {
            Ok(chapter) => Some(Set::from_lines(chapter, &set_lines)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R: BufRead> Iterator for Iter<R> {
    type Item = Result<Set, RError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.format {
            Format::Reaclib1 => self.next_v1(),
            Format::Reaclib2 => self.next_v2(),
        }
    }
}

/// Get a [`HashMap`] mapping reactions to a [`Vec`] of [`Set`]s.
///
/// This is useful because multiple `Set`s may be needed to describe a reaction rate.
///
/// # Examples
///
/// ```
/// use reaclib::{to_hash_map, Format};
/// use std::io;
///
/// let stdin = io::stdin().lock();
/// let map = to_hash_map(stdin, Format::Reaclib1).unwrap();
/// ```
///
/// # Errors
///
/// Will return `Err` if there is an io error or a parsing error.
pub fn to_hash_map<R: BufRead>(
    reader: R,
    format: Format,
) -> Result<HashMap<Reaction, Vec<Set>>, RError> {
    let mut m = HashMap::new();

    for set in Iter::new(reader, format) {
        let set = set?;
        let key = (set.reactants.clone(), set.products.clone());
        m.entry(key).or_insert_with(Vec::new).push(set);
    }

    Ok(m)
}
