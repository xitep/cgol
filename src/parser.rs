use std::fmt;
use std::fs::{File};
use std::io::{Read};

use bit_vec::BitVec;
use world::World;

/// Loads a world from the given filename. Results either in the
/// loaded world or a human readable error description.
pub fn load_from_file(filename: &str) -> Result<World, String> {
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
    let w = err!(parse(&s));
    Ok(w)
}

// --------------------------------------------------------------------

#[derive(Debug)]
struct Error {
    row: usize,
    col: usize,
    reason: String
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}:{}: {}", self.row, self.col, self.reason)
    }
}

// --------------------------------------------------------------------

fn parse(world: &str) -> Result<World, Error> {
    let mut cells = BitVec::new();
    let (mut width, mut height) = (0usize, 0usize);

    let (mut row_no, mut col_no) = (0usize, 0usize);
    // ~ for each row of the world
    for row in world.lines() {
        row_no += 1;
        col_no = 0usize;

        let mut curr_width = 0usize;

        macro_rules! check_row_too_long {
            () => {{
                if width > 0 && curr_width >= width {
                    return Err(Error{
                        row: row_no,
                        col: col_no,
                        reason: "Row too long".into(),
                    });
                } else {
                    curr_width += 1;
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
                c => return Err(Error{
                    row: row_no,
                    col: col_no,
                    reason: format!("Invalid character: {}", c),
                }),
            }
        }
        trace!("validating row ({}) with the previous one", row_no);
        // ~ check the with of the current row is the same as of
        // the previous row
        if width > 0 {
            if curr_width != width {
                return Err(Error{
                    row: row_no,
                    col: col_no,
                    reason: "Row too short".into(),
                });
            }
        } else {
            width = curr_width;
        }
        // ~ track height of the world (ignoring leading empty lines)
        if width > 0 {
            height += 1;
        }
    }
    if width == 0 || height == 0 {
        return Err(Error{
            row: row_no,
            col: col_no,
            reason: "Empty world!".into()
        });
    }
    cells.shrink_to_fit();

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
