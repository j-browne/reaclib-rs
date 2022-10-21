use crate::{error::ReaclibError, Format, Iter};
use std::io::Cursor;

// if the file is empty, that's not an error, there are just no items
#[test]
fn empty() {
    let reader = Cursor::new(include_str!("v1/empty"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().is_none());
}

#[test]
fn single() {
    let reader = Cursor::new(include_str!("v1/single"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

// without a chapter, we don't know how to interpret the nuclide list, so it is an error
#[test]
fn chapter_unset() {
    let reader = Cursor::new(include_str!("v1/chapter_unset"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::ChapterUnset));
    assert!(iter.next().is_none());
}

// this is making sure that there's no problem with switching between chapters.
// since we don't explicitly keep track of the chapter, we check the number of reactants and
// products
// also, make sure that there's no problem with empty chapters
#[test]
fn multi_chapter() {
    let reader = Cursor::new(include_str!("v1/multi_chapter"));
    let mut iter = Iter::new(reader, Format::Reaclib1);

    let set = iter.next().unwrap().unwrap();
    assert_eq!(set.reactants.len(), 1);
    assert_eq!(set.products.len(), 4);
    let set = iter.next().unwrap().unwrap();
    assert_eq!(set.reactants.len(), 1);
    assert_eq!(set.products.len(), 4);
    let set = iter.next().unwrap().unwrap();
    assert_eq!(set.reactants.len(), 1);
    assert_eq!(set.products.len(), 4);
    let set = iter.next().unwrap().unwrap();
    assert_eq!(set.reactants.len(), 1);
    assert_eq!(set.products.len(), 1);
    let set = iter.next().unwrap().unwrap();
    assert_eq!(set.reactants.len(), 2);
    assert_eq!(set.products.len(), 4);
    let set = iter.next().unwrap().unwrap();
    assert_eq!(set.reactants.len(), 2);
    assert_eq!(set.products.len(), 4);
    assert!(iter.next().is_none());
}

// make sure you get the right error when the line is too short
// it is not an error if spaces are left off the end
#[test]
fn unfinished_line() {
    // the end spaces don't matter
    let reader = Cursor::new(include_str!("v1/unfinished_line_1"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());

    // the end spaces don't matter
    let reader = Cursor::new(include_str!("v1/unfinished_line_2"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());

    // the end spaces don't matter
    let reader = Cursor::new(include_str!("v1/unfinished_line_3"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());

    let reader = Cursor::new(include_str!("v1/unfinished_line_4"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooShortLine));
    assert!(iter.next().is_none());

    let reader = Cursor::new(include_str!("v1/unfinished_line_5"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooShortLine));
    assert!(iter.next().is_none());

    // the end spaces don't matter
    let reader = Cursor::new(include_str!("v1/unfinished_line_6"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());
}

// this is for if a set doesn't have all three lines
#[test]
fn too_few_lines() {
    let reader = Cursor::new(include_str!("v1/too_few_lines"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().unwrap().is_ok());
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooFewLines));
    assert!(iter.next().is_none());
}

// the input for this test has multi-byte chars
#[test]
fn str_index() {
    // the char spans a slice boundary, so we get an indexing error
    let reader = Cursor::new(include_str!("v1/str_index_1"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::StrIndex));
    assert!(iter.next().is_none());

    // the char is within a slice, so we get a parsing error
    let reader = Cursor::new(include_str!("v1/str_index_2"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseFloat(_))
    ));
    assert!(iter.next().is_none());
}

#[test]
fn unknown_chapter() {
    let reader = Cursor::new(include_str!("v1/unknown_chapter"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(iter.next().unwrap().is_ok());
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::UnknownChapter(12)));
}

#[test]
fn unknown_resonance() {
    use crate::Resonance as R;

    let reader = Cursor::new(include_str!("v1/unknown_resonance"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert_eq!(iter.next().unwrap().unwrap().resonance, R::Resonant);
    assert_eq!(iter.next().unwrap().unwrap().resonance, R::NonResonant);
    assert_eq!(iter.next().unwrap().unwrap().resonance, R::NonResonant);
    assert_eq!(iter.next().unwrap().unwrap().resonance, R::Weak);
    assert_eq!(iter.next().unwrap().unwrap().resonance, R::S);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::UnknownResonance(_))
    ));
}

// This test should be able to open "src", but since it is a directory, reading from it should be
// an error.
#[test]
fn io_error() {
    use std::{fs::File, io::BufReader};

    let reader = File::open("src").unwrap();
    let reader = BufReader::new(reader);
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(iter.next().unwrap(), Err(ReaclibError::Io(_))));
}

#[test]
fn parse_error() {
    // fails to parse an int in a chapter
    let reader = Cursor::new(include_str!("v1/parse_int_error"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseInt(_))
    ));

    // fails to parse a float in the q-value
    let reader = Cursor::new(include_str!("v1/parse_float_error_1"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseFloat(_))
    ));

    // fails to parse a float in the params
    let reader = Cursor::new(include_str!("v1/parse_float_error_2"));
    let mut iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseFloat(_))
    ));
}

#[test]
fn multi() {
    let reader = Cursor::new(include_str!("v1/multi"));
    let iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(iter.collect::<Result<Vec<_>, _>>(), Ok(_)));
}

// This should fail when trying to parse a v2 file
#[test]
fn multi_v2() {
    let reader = Cursor::new(include_str!("v2/multi"));
    let iter = Iter::new(reader, Format::Reaclib1);
    assert!(matches!(iter.collect::<Result<Vec<_>, _>>(), Err(_)));
}
