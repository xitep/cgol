use std::cmp;
use std::fmt;
use std::fs::{File};
use std::io::{Read};
use std::iter;

use bit_vec::BitVec;
use world::World;

/// Loads a world from the given filename. Results either in the
/// loaded world or a human readable error description.
pub fn load_from_file(min_width: usize, min_height: usize, filename: &str)
                      -> Result<World, String>
{
    macro_rules! err {
        ($expr:expr) => {
            match $expr {
                Err(e) => return Err(format!("{}: {}", filename, e)),
                Ok(v) => v,
            }
        }
    }
    let mut f = err!(File::open(filename));
    let mut s = String::with_capacity(err!(f.metadata()).len() as usize);
    err!(f.read_to_string(&mut s));
    let w = err!(parse(min_width, min_height, &s));
    Ok(w)
}

// --------------------------------------------------------------------

#[derive(Debug)]
struct ParseError {
    row: usize,
    col: usize,
    reason: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}:{}: {}", self.row, self.col, self.reason)
    }
}

// --------------------------------------------------------------------

fn parse(min_width: usize, min_height: usize, world: &str)
         -> Result<World, ParseError>
{
    let mut cells = BitVec::new();
    let (mut map_width, mut map_height) = (0usize, 0usize);

    let (mut row_no, mut col_no) = (0usize, 0usize);
    // ~ for each row of the world
    for row in world.lines() {
        row_no += 1;
        col_no = 0usize;

        let mut curr_map_width = 0usize;

        macro_rules! check_row_too_long {
            () => {{
                if map_width > 0 && curr_map_width >= map_width {
                    return Err(ParseError{
                        row: row_no,
                        col: col_no,
                        reason: "Row too long".into(),
                    });
                } else {
                    curr_map_width += 1;
                }
            }}
        }

        // ~ for each cell of the current row
        for c in row.chars() {
            trace!("{}:{}: {} ({:X})", row_no, col_no, c, c as u32);
            col_no += 1;
            match c {
                ' ' => {
                    /* ignore (only) blanks */
                }
                '.' => {
                    check_row_too_long!();
                    cells.push(false);
                }
                '#' => {
                    check_row_too_long!();
                    cells.push(true);
                }
                // everything else: an invalid input
                c => return Err(ParseError{
                    row: row_no,
                    col: col_no,
                    reason: format!("Invalid character: {}", c),
                }),
            }
        }
        trace!("validating row ({}) with the previous one", row_no);
        // ~ check the with of the current row is the same as of
        // the previous row
        if map_width > 0 {
            if curr_map_width != map_width {
                return Err(ParseError{
                    row: row_no,
                    col: col_no,
                    reason: "Row too short".into(),
                });
            }
        } else {
            map_width = curr_map_width;
        }
        // ~ track height of the world (ignoring leading empty lines)
        if map_width > 0 {
            if map_width < min_width {
                for _ in map_width..min_width {
                    cells.push(false);
                }
            }
            map_height += 1;
        }
    }
    if map_width == 0 || map_height == 0 {
        return Err(ParseError{
            row: row_no,
            col: col_no,
            reason: "Empty world!".into()
        });
    }
    if map_height < min_height {
        let width = cmp::max(map_width, min_width);
        for _ in map_height..min_height {
            cells.extend(iter::repeat(false).take(width));
        }
    }
    cells.shrink_to_fit();

    let width = cmp::max(map_width, min_width);
    let height = cmp::max(map_height, min_height);
    debug_assert!(width*height == cells.len());
    Ok(World::new(width, height, cells).unwrap())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_parse_empty() {
        assert!(parse(0, 0, "").is_err());
    }

    #[test]
    fn test_parse_oneline() {
        // XXX shouldn't this actually constitute an invalid world?
        let r = parse(0, 0, ".#.");
        println!("{:?}", r);
        assert!(r.is_ok());
    }

    #[test]
    fn test_parse_twolines() {
        let r = parse(0, 0, ".#.\n.#.");
        println!("{:?}", r);
        assert!(r.is_ok());
    }

    #[test]
    fn test_parse_blinker() {
        let r = parse(0, 0, r#"
. . .
. # .
. # .
. # .
. . .
"#);
        println!("{:?}", r);
        assert!(r.is_ok());
    }

}
