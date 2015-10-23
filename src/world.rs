use std::fmt::{self, Write};

use rand::Rng;

pub struct World {
    width: usize,
    height: usize,

    generation: usize, // current generation of cells
    alive: usize, // current number of live cells
    cells: Vec<u8>,   // cells addressable by: `x + y*width`; 1 if alive, 0 if dead
}

impl fmt::Debug for World {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt,
               "{}x{} (alive: {})",
               self.width,
               self.height,
               self.alive)
    }
}

impl World {
    pub fn empty(width: usize, height: usize) -> World {
        World::from_cells(width, height, vec![false; width * height])
    }

    pub fn random<R: Rng>(r: &mut R, width: usize, height: usize) -> World {
        let mut v = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            v.push(r.gen::<f64>() < 0.3);
        }
        World::from_cells(width, height, v)
    }

    fn from_cells(width: usize, height: usize, cells: Vec<bool>) -> World {
        assert_eq!(width * height, cells.len());
        World {
            width: width,
            height: height,
            generation: 0,
            alive: cells.iter().filter(|&x| *x).count(),
            cells: cells.into_iter().map(|x| if x { 1 } else { 0 }).collect(),
        }
    }

    #[inline]
    pub fn alive(&self) -> usize {
        self.alive
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn expand_to(&mut self, new_width: usize, new_height: usize) {
        if new_width == self.width && new_height == self.height {
            return;
        }
        let mut alive = 0;
        let mut ncells = vec![0; new_width * new_height];
        for h in 0..self.height {
            for w in 0..self.width {
                if self.is_alive(w, h) {
                    let (nw, nh) = (w % new_width, h % new_height);
                    *ncells.get_mut(nh * new_width + nw).unwrap() = 1;
                    alive += 1;
                }
            }
        }
        self.width = new_width;
        self.height = new_height;
        self.cells = ncells;
        self.alive = alive;
    }

    pub fn set_alive(&mut self, w: usize, h: usize, alive: bool) {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        unsafe { *self.cells.get_unchecked_mut(h * self.width + w) = if alive { 1 } else { 0 } };
        if alive {
            self.alive += 1;
        } else {
            self.alive -= 1;
        }
    }

    pub fn is_alive(&self, w: usize, h: usize) -> bool {
        self.is_alive_num(w, h) != 0
    }

    // ~ returns 1 if the specified cell is alive, otherwise 0.
    fn is_alive_num(&self, w: usize, h: usize) -> usize {
        self.cell(self.cell_offset(w, h)) as usize
    }

    #[inline]
    fn cell_offset(&self, w: usize, h: usize) -> usize {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);
        h * self.width + w
    }

    #[inline]
    fn cell(&self, offset: usize) -> u8 {
        debug_assert!(offset < self.cells.len());
        unsafe { *self.cells.get_unchecked(offset) }
    }

    //
    // Rules (https://en.wikipedia.org/wiki/Conway's_Game_of_Life):
    //
    // 1. Any live cell with fewer than two live neighbours dies, as if
    // caused by under-population.
    // 2. Any live cell with two or three live neighbours lives on to the
    // next generation.
    // 3. Any live cell with more than three live neighbours dies, as if
    // by over-population.
    // 4. Any dead cell with exactly three live neighbours becomes a live
    // cell, as if by reproduction.
    //
    pub fn advance_generation<F: FnMut(usize, usize, bool)>(&mut self, mut cb: F) {

        // ~ computes the the number of alive neighbours for (w, h)
        // assuming the cell is somewhere at the border of the world.
        fn border_neighbour_count(world: &World, w: usize, h: usize) -> (bool, usize) {
            let mut cnt =
                  world.is_alive_num(wrapped(w, -1, world.width), wrapped(h, -1, world.height))
                + world.is_alive_num(w,                           wrapped(h, -1, world.height))
                + world.is_alive_num(wrapped(w, 1, world.width),  wrapped(h, -1, world.height));

            cnt += world.is_alive_num(wrapped(w, -1, world.width), h)
                + world.is_alive_num(wrapped(w, 1, world.width),  h);

            cnt += world.is_alive_num(wrapped(w, -1, world.width), wrapped(h, 1, world.height))
                + world.is_alive_num(w,                           wrapped(h, 1, world.height))
                + world.is_alive_num(wrapped(w, 1, world.width),  wrapped(h, 1, world.height));

            (world.is_alive(w, h), cnt)
        }

        // ~ computes the number of alive neighbours for (w, h)
        // assuming the cell is not at the border of the world.
        fn inner_neighbour_count(world: &World, w: usize, h: usize) -> (bool, usize) {
            let mut cnt =
                  world.is_alive_num(w - 1, h - 1)
                + world.is_alive_num(w,     h - 1)
                + world.is_alive_num(w + 1, h - 1);

            cnt += world.is_alive_num(w - 1, h)
                + world.is_alive_num(w + 1, h);

            let center_alive = world.is_alive(w, h);

            cnt += world.is_alive_num(w - 1, h + 1)
                + world.is_alive_num(w,     h + 1)
                + world.is_alive_num(w + 1, h + 1);

            (center_alive, cnt)
        }

        let mut changes = Vec::new();
        macro_rules! eval_counts {
            ($w:expr, $h:expr, $count:expr) => {
                match $count {
                    (true, n) if n < 2 || n > 3 => changes.push(($w, $h, false)),
                    (false, 3) => changes.push(($w, $h, true)),
                    _ => {}
                }
            }
        }

        {
            // upper row
            for w in 0..self.width {
                eval_counts!(w, 0, border_neighbour_count(self, w, 0))
            }
            // lower row
            let h = self.height - 1;
            for w in 0..self.width {
                eval_counts!(w, h, border_neighbour_count(self, w, h))
            }
            // left column
            for h in 1..(self.height - 1) {
                eval_counts!(0, h, border_neighbour_count(self, 0, h))
            }
            // right column
            let w = self.width - 1;
            for h in 1..(self.height - 1) {
                eval_counts!(w, h, border_neighbour_count(self, w, h))
            }
        }
        // inner cells
        for h in 1..(self.height - 1) {
            for w in 1..(self.width -1) {
                eval_counts!(w, h, inner_neighbour_count(self, w, h));
            }
        }
        // apply changes
        for &(w, h, change) in changes.iter() {
            self.set_alive(w, h, change);
        }
        // track the number of generations advanced
        self.generation += 1;
        // notify callback
        for &(w, h, change) in changes.iter() {
            cb(w, h, change);
        }
    }
}

fn wrapped(w: usize, offs: isize, wrap: usize) -> usize {
    let (w, wrap) = (w as isize, wrap as isize);
    let n = (w + offs) % wrap;
    if n < 0 {
        (wrap + n) as usize
    } else {
        n as usize
    }
}

#[test]
fn test_wrapped() {
    assert_eq!(wrapped(1, 1, 5), 2);
    assert_eq!(wrapped(1, -1, 5), 0);
    assert_eq!(wrapped(0, -1, 5), 4);
    assert_eq!(wrapped(0, -2, 5), 3);
    assert_eq!(wrapped(0, -11, 5), 4);
    assert_eq!(wrapped(0, -10, 5), 0);
    assert_eq!(wrapped(1, -11, 5), 0);
    assert_eq!(wrapped(2, -11, 5), 1);
}

#[cfg(test)]
mod benches {
    use super::World;

    use rand::XorShiftRng;
    use test::{black_box, Bencher};

    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    #[bench]
    fn advance_generation_random_world(b: &mut Bencher) {
        let mut w = World::random(&mut XorShiftRng::new_unseeded(), WIDTH, HEIGHT);
        b.iter(|| w.advance_generation(|_, _, state| {
            black_box(state);
        }));
    }
}
