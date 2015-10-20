use std::fmt::{self, Write};

use rustbox::{self, RustBox, Event, Color};
use rustbox::keyboard::{Key};
use world::World;

pub fn run(world: World) -> Result<(), String> {
    run_(world).map_err(|e| format!("error: {}", e))
}

enum Error {
    RustboxInit(rustbox::InitError),
    RustboxEvent(rustbox::EventError),
}

impl From<rustbox::InitError> for Error {
    fn from(e: rustbox::InitError) -> Self { Error::RustboxInit(e) }
}
impl From<rustbox::EventError> for Error {
    fn from(e: rustbox::EventError) -> Self { Error::RustboxEvent(e) }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::RustboxEvent(ref e) =>
                write!(fmt, "rustbox::event: {}", e),
            Error::RustboxInit(ref e) =>
                write!(fmt, "rustbox::init: {}", e),
        }
    }
}

struct UI {
    terminal: RustBox,
    width:    usize,
    height:   usize,

    line_buf: String,
}

impl UI {
    fn init() -> Result<UI, Error> {
        let t = try!(RustBox::init(Default::default()));
        let (width, height) = (t.width(), t.height());
        Ok(UI {
            terminal: t,
            width: width,
            height: height,
            line_buf: String::with_capacity(width),
        })
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn print_world(&mut self, world: &World) {
        for h in 0..world.height() {
            self.line_buf.clear();
            world.render_line(h, &mut self.line_buf);
            self.print_line(0, h, &self.line_buf);
        }
        self.print_status(format_args!("Gen: {}", world.generation()));
    }

    fn print_status(&mut self, args: fmt::Arguments) {
        self.line_buf.clear();
        let _ = self.line_buf.write_fmt(args);
        self.print_line(0, self.height - 1, &self.line_buf);
    }

    fn print_line(&self, x: usize, y: usize, line: &str) {
        self.terminal.print(x, y, rustbox::RB_NORMAL, Color::Default, Color::Default, line);
    }

    fn clear(&self) {
        self.terminal.clear();
    }

    fn flush(&self) {
        self.terminal.present();
    }
}

fn run_(mut world: World) -> Result<(), Error> {
    let mut ui = try!(UI::init());
    ui.print_world(&world);
    ui.flush();

    loop {
        match try!(ui.terminal.poll_event(false)) {
            Event::KeyEvent(Some(key)) => {
                match key {
                    Key::Char('q') => break,
                    Key::Char(' ') => {
                        world.advance_generation();
                        ui.print_world(&world);
                        ui.flush();
                    }
                    _ => {},
                }
            }
            Event::ResizeEvent(w, h) => {
                ui.set_size(w.abs() as usize, h.abs() as usize);
                ui.clear();
                ui.print_world(&world);
                ui.flush();
            }
            _ => {},
        }
    }
    Ok(())
}
