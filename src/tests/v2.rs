use crate::{error::ReaclibError, Format, Iter};
use std::io::{self, Cursor};

// if the file is empty, that's not an error, there are just no items
#[test]
fn empty() {
    let reader = Cursor::new(include_str!("v2/empty"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(iter.next().is_none());
}

#[test]
fn single() {
    let reader = Cursor::new(include_str!("v2/single"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}

// make sure you get the right error when the line is too short
// it is not an error if spaces are left off the end
#[test]
fn unfinished_line() {
    // the end spaces don't matter
    let reader = Cursor::new(include_str!("v2/unfinished_line_1"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());

    let reader = Cursor::new(include_str!("v2/unfinished_line_2"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooShortLine));
    assert!(iter.next().is_none());

    let reader = Cursor::new(include_str!("v2/unfinished_line_3"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooShortLine));
    assert!(iter.next().is_none());

    // the end spaces don't matter
    let reader = Cursor::new(include_str!("v2/unfinished_line_4"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());
}

// this is for if a set doesn't have all four lines
#[test]
fn too_few_lines() {
    // if we don't have a chapter
    let reader = Cursor::new(include_str!("v2/too_few_lines_1"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooFewLines));
    assert!(iter.next().is_none());

    // if we don't have the rest of a set
    let reader = Cursor::new(include_str!("v2/too_few_lines_2"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(iter.next().unwrap().is_ok());
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::TooFewLines));
    assert!(iter.next().is_none());
}

// the input for this test has multi-byte chars
#[test]
fn str_index() {
    // the char spans a slice boundary, so we get an indexing error
    let reader = Cursor::new(include_str!("v2/str_index_1"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::StrIndex));
    assert!(iter.next().is_none());

    // the char is within a slice, so we get a parsing error
    let reader = Cursor::new(include_str!("v2/str_index_2"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseFloat(_))
    ));
    assert!(iter.next().is_none());
}

// the input for this test has a non-utf8 byte
#[test]
fn non_utf8() {
    let reader = Cursor::new(include_bytes!("v2/non_utf8"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert_eq!(
        iter.next().unwrap(),
        Err(ReaclibError::Io(io::ErrorKind::InvalidData))
    );
}

#[test]
fn unknown_chapter() {
    let reader = Cursor::new(include_str!("v2/unknown_chapter"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(iter.next().unwrap().is_ok());
    assert_eq!(iter.next().unwrap(), Err(ReaclibError::UnknownChapter(12)));
}

#[test]
fn unknown_resonance() {
    use crate::Resonance as R;

    let reader = Cursor::new(include_str!("v2/unknown_resonance"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
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
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(iter.next().unwrap(), Err(ReaclibError::Io(_))));
}

#[test]
fn parse_error() {
    // without a chapter line, it will try to interpret the first line as a chapter
    let reader = Cursor::new(include_str!("v2/parse_int_error_1"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseInt(_))
    ));

    // fails to parse an int in a chapter
    let reader = Cursor::new(include_str!("v2/parse_int_error_2"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseInt(_))
    ));

    // fails to parse a float in the q-value
    let reader = Cursor::new(include_str!("v2/parse_float_error_1"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseFloat(_))
    ));

    // fails to parse a float in the params
    let reader = Cursor::new(include_str!("v2/parse_float_error_2"));
    let mut iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(
        iter.next().unwrap(),
        Err(ReaclibError::ParseFloat(_))
    ));
}

#[test]
fn multi() {
    let reader = Cursor::new(include_str!("v2/multi"));
    let iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(iter.collect::<Result<Vec<_>, _>>(), Ok(_)));
}

// This should fail when trying to parse a v1 file
#[test]
fn multi_v2() {
    let reader = Cursor::new(include_str!("v1/multi"));
    let iter = Iter::new(reader, Format::Reaclib2);
    assert!(matches!(iter.collect::<Result<Vec<_>, _>>(), Err(_)));
}
