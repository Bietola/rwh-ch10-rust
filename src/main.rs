use std::error::Error;
use std::io::Read;

struct PGM {
    width: usize,
    height: usize,
    max_grey_val: u8,
    contents: Vec<bool>,
}

type ParseInput<'a> = &'a [u8];
type ParseResult<'a, T> = Result<(T, ParseInput<'a>), &'static str>;

fn parse_pgm(input: ParseInput) -> ParseResult<PGM> {
    let (_, rest) = match_header(input)?;

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

fn match_header(input: ParseInput) -> ParseResult<()> {
    if input.split("hello") {

    }
}

fn get_num(input: ParseInput) -> ParseResult<i32> {
    unimplemented!()
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
