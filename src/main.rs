#![feature(result_flattening)]
#![feature(trace_macros)]

use bstr::ByteSlice;
use newtype::NewType;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::io::Read;

/********************************************/
/* Parser general definition and properties */
/********************************************/

trait Parser<'a, Out> {
    fn parse(&self, input: ParseInput<'a>) -> ParseResult<'a, Out>;
}

impl<'a, Out, F> Parser<'a, Out> for F
where
    F: Fn(ParseInput<'a>) -> ParseResult<Out>,
{
    fn parse(&self, input: ParseInput<'a>) -> ParseResult<'a, Out> {
        self(input)
    }
}

fn map<'a, Out, Out1, F, P>(parser: P, fun: F) -> impl Parser<'a, Out1>
where
    F: Fn(Out) -> Out1,
    P: Parser<'a, Out>,
{
    move |input| parser.parse(input).map(|(out, rest)| (fun(out), rest))
}

fn and_then<'a, Out, Out1, F, P, P1>(parser: P, fun: F) -> impl Parser<'a, Out1>
where
    F: Fn(Out) -> P1,
    P: Parser<'a, Out>,
    P1: Parser<'a, Out1>,
{
    move |input| {
        parser
            .parse(input)
            .and_then(|(out, rest)| fun(out).parse(rest))
    }
}

#[derive(Debug, PartialEq)]
enum ParseErr {
    NoValidFieldLeft,
    NoHeaderMatch,
    Utf8Error(bstr::Utf8Error),
    InvalidNum(std::num::ParseIntError),
    InvByte(String), // Ugly hack to permit derivation of `PartialEq`
}

macro_rules! parse_do {
    (return $val:expr,) => {
        move |input| Ok(($val, input))
    };
    ($out:tt <- $parser:expr, $($tail:tt)*) => {
        and_then($parser, move |$out| parse_do!($($tail)*))
    };
    ($parser:tt, $($tail:tt)*) => {
        and_then($parser, move |_| parse_do!($($tail)*))
    };
}

/****************/
/* PGM Datatype */
/****************/

#[derive(Debug)]
struct PGM {
    width: usize,
    height: usize,
    max_grey_val: u8,
    contents: Contents,
}

#[derive(NewType)]
struct Contents(Vec<u8>);

impl fmt::Debug for Contents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}...<contents>", &self.0[0..20].as_bstr())
    }
}

type ParseInput<'a> = &'a [u8];
type ParseResult<'a, T> = Result<(T, ParseInput<'a>), (ParseErr, ParseInput<'a>)>;

/*******************/
/* Parser builders */
/*******************/

fn parse_pgm(input: ParseInput) -> ParseResult<PGM> {
    let parser = parse_do! {
        match_header_version,
        width <- get_num,
        height <- get_num,
        max_grey_val <- get_num,
        contents <- move |i| get_bytes(i, (width * height) as usize),

        return PGM {
            width: width as usize,
            height: height as usize,
            max_grey_val: max_grey_val as u8,
            contents: contents.into(),
        },
    };

    parser.parse(input)
}

fn match_header_version(input: ParseInput) -> ParseResult<()> {
    const VERSION_STR: &str = "P5";

    if input.starts_with_str(VERSION_STR) {
        // +1 is for the `\n` after the VERSION_STR
        let read_until = VERSION_STR.len() + 1;

        Ok(((), &input[read_until..]))
    } else {
        Err((ParseErr::NoHeaderMatch, input))
    }
}

fn get_num(input: ParseInput) -> ParseResult<i32> {
    let raw_num_str = input
        .fields()
        .next()
        .ok_or_else(|| (ParseErr::NoValidFieldLeft, input))?;

    let num = raw_num_str.to_str().map_or_else(
        |er| Err((ParseErr::Utf8Error(er), input)),
        |s| {
            s.parse::<i32>()
                .or_else(|er| Err((ParseErr::InvalidNum(er), input)))
        },
    )?;

    // `parsed_len` is length to consume after parse. The comparison is for "end of string" edge
    // case.
    let len = raw_num_str.len();
    let parsed_len = match len.cmp(&input.len()) {
        Ordering::Greater => panic!("Paradoxically parsed beyond string end"),
        Ordering::Equal => len,
        // +1 for skipping next whitespace (there should be always one is PGM specification)
        Ordering::Less => len + 1,
    };

    Ok((num, &input[parsed_len..]))
}

fn get_bytes(input: ParseInput, amount: usize) -> ParseResult<Vec<u8>> {
    let parsed = <ParseInput as std::io::Read>::bytes(input)
        .take(amount)
        .fold(Ok(vec![]), |s, e| {
            s.map(|mut s| {
                e.map(|e| {
                    s.push(e);
                    s
                })
            })
            .flatten()
        })
    .or_else(|er| Err((ParseErr::InvByte(er.to_string()), input)))?;

    Ok((parsed, &input[amount..]))
}

/********/
/* Main */
/********/

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::open("assets/lolcat.pgm")?;

    let mut contents = vec![];
    file.read_to_end(&mut contents)?;

    let contents = bstr::BString::from(contents);
    let pgm = parse_pgm(contents.as_slice());

    println!("{:?}", pgm);

    Ok(())
}

/*********/
/* Tests */
/*********/

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn match_header_version_obv() {
        let mock_header: &[u8] = indoc!(
            "P5
            120 32"
        )
            .as_bytes();

        assert_eq!(
            match match_header_version(mock_header) {
                Ok(((), s)) => Ok(((), s.as_bstr())),
                Err((er, s)) => Err((er, s.as_bstr())),
            },
            Ok(((), "120 32".as_bytes().as_bstr())),
        );
    }

    #[test]
    fn get_num_single() {
        assert_eq!(get_num("12".as_bytes()), Ok((12, "".as_bytes())));
    }

    #[test]
    fn get_num_multiple() {
        assert_eq!(get_num("12 24".as_bytes()), Ok((12, "24".as_bytes())));
    }

    #[test]
    fn and_then_map_2_nums() {
        let input = b"12 14 16";

        let res = and_then(get_num, move |n1| map(get_num, move |n2| (n1, n2))).parse(input);

        assert_eq!(res, Ok(((12, 14), "16".as_bytes())),);
    }
}
