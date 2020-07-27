use bstr::ByteSlice;
use std::cmp::Ordering;
use std::error::Error;
use std::io::Read;

#[derive(PartialEq, Debug)]
enum ParseErr {
    NoValidFieldLeft,
    NoHeaderMatch,
    Utf8Error(bstr::Utf8Error),
    InvalidNum(std::num::ParseIntError),
}

struct PGM {
    width: usize,
    height: usize,
    max_grey_val: u8,
    contents: Vec<bool>,
}

type ParseInput<'a> = &'a [u8];
type ParseResult<'a, T> = Result<(T, ParseInput<'a>), (ParseErr, ParseInput<'a>)>;

fn parse_pgm(input: ParseInput) -> ParseResult<PGM> {
    let (_, rest) = match_header_version(input)?;

    let (width, rest) = get_num(rest)?;
    let (height, rest) = get_num(rest)?;
    let (max_grey_val, rest) = get_num(rest)?;
    let (contents, rest) = get_bits(rest, width * height)?;

    Ok((
        PGM {
            width: width as usize,
            height: height as usize,
            max_grey_val: max_grey_val as u8,
            contents,
        },
        rest,
    ))
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

fn get_bits(input: ParseInput, amount: i32) -> ParseResult<Vec<bool>> {
    unimplemented!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::open("assets/lolcat.pgm")?;

    let mut contents = vec![];
    file.read_to_end(&mut contents)?;

    let pgm = parse_pgm(bstr::BString::from(contents).as_slice());

    Ok(())
}

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
}
