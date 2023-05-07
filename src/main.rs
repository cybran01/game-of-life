#![windows_subsystem = "windows"]

use fltk::{
    app,
    app::remove_timeout3,
    button::{Button, CheckButton, ToggleButton},
    enums::{FrameType, Shortcut},
    input::FloatInput,
    menu::{Choice, MenuFlag},
    prelude::WidgetExt,
    prelude::*,
    text::TextDisplay,
    window::Window,
};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub mod canvas;
pub mod field;
use crate::canvas::Canvas;
use crate::field::Shape;

const WIDTH: i32 = 800 * 2;
const HEIGHT: i32 = 600 * 2;
const TICKTIME: f64 = 0.05;
const STARTLINEDIST: i32 = 30;
const INITIALUPDATEINTERVALL: f64 = 0.1;
const XSTARTOFFSET: i32 = 0;
const YSTARTOFFSET: i32 = 0;
const CHUNKSIZE: usize = 8;

//helper function for rotating double vecs
fn mirror_diag<T: Copy>(vec: &mut Vec<Vec<Option<T>>>) {
    let mut rotshape = Vec::new();
    let vect_maxlen = vec.iter().fold(0, |x, y| std::cmp::max(x, y.len()));

    for i in 0..vect_maxlen {
        let mut curcolumn = Vec::new();
        let shapeiter = vec.iter();
        for j in shapeiter {
            curcolumn.push(*j.get(i).unwrap_or(&None));
        }
        rotshape.push(curcolumn);
    }
    *vec = rotshape;
}

//helper function turning an iterator of results into a result that either is the iterator over the non-error type or the first encounterd error
fn return_first_err<T, E>(
    vec: impl Iterator<Item = Result<T, E>>,
) -> Result<impl Iterator<Item = T>, E> {
    let mut res = Vec::new();

    for x in vec.into_iter() {
        match x {
            Ok(x) => res.push(x),
            Err(err) => return Err(err),
        }
    }

    Ok(res.into_iter())
}

fn parse_file(file: &PathBuf) -> Option<Shape> {
    //Reads entire file into buffer, not a great idea for huge files
    let bytebuf: Vec<u8> = std::fs::read(file)
        .unwrap_or_else(|_| panic!("file read error at {}", file.to_str().unwrap_or("")));
    let mut curshape = Vec::new();

    let bytebuflines = bytebuf
        .split(|b| *b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line));

    for line in bytebuflines {
        let mut curline = Vec::new();
        for b in line {
            if *b == b'0' {
                curline.push(Some(false));
            } else if *b == b'1' {
                curline.push(Some(true));
            } else {
                curline.push(None);
            }
        }
        curshape.push(curline);
    }
    Some(curshape)
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Game of Life");

    let canvas = Canvas::new(
        WIDTH,
        HEIGHT,
        CHUNKSIZE,
        XSTARTOFFSET,
        YSTARTOFFSET,
        STARTLINEDIST,
    );
    wind.add(&*canvas);
    wind.make_resizable(true);

    let btn_step = Button::default().with_label("Step");
    wind.add(&btn_step);

    let btn_clear = Button::default().with_label("Clear");
    wind.add(&btn_clear);

    let mut btn_stop_toggle = ToggleButton::default().with_label("Stop");
    btn_stop_toggle.set_value(true);
    btn_stop_toggle.set_shortcut(fltk::enums::Shortcut::Alt);
    wind.add(&btn_stop_toggle);

    let btn_drawchunks = CheckButton::default().with_label("Draw chunks");
    wind.add(&btn_drawchunks);

    let mnu_shapeselect = Choice::default().with_label("Insert shape:");
    wind.add(&mnu_shapeselect);

    let mut btn_mirror_shape = Button::default().with_label("Flip");

    btn_mirror_shape.deactivate();
    wind.add(&btn_mirror_shape);

    let mut btn_rotate_shape = Button::default().with_label("Rotate");
    btn_rotate_shape.deactivate();
    wind.add(&btn_rotate_shape);

    let mut inp_update_intervall = FloatInput::default().with_label("Update intervall:");
    inp_update_intervall.set_value(format!("{}", INITIALUPDATEINTERVALL).as_str());
    wind.add(&inp_update_intervall);

    let mut lbl_coords = TextDisplay::default();
    lbl_coords.set_frame(FrameType::NoBox); //<- i hate this
    wind.add(&lbl_coords);

    let canvas = Rc::new(RefCell::new(canvas));
    let btn_stop_toggle = Rc::new(RefCell::new(btn_stop_toggle));
    let btn_drawchunks = Rc::new(RefCell::new(btn_drawchunks));
    let mnu_shapeselect = Rc::new(RefCell::new(mnu_shapeselect));
    let btn_step = Rc::new(RefCell::new(btn_step));
    let btn_clear = Rc::new(RefCell::new(btn_clear));
    let btn_mirror_shape = Rc::new(RefCell::new(btn_mirror_shape));
    let btn_rotate_shape: Rc<RefCell<Button>> = Rc::new(RefCell::new(btn_rotate_shape));
    let inp_update_intervall = Rc::new(RefCell::new(inp_update_intervall));
    let lbl_coords = Rc::new(RefCell::new(lbl_coords));

    {
        let canvas = canvas.clone();
        let btn_stop_toggle = btn_stop_toggle.clone();
        let btn_drawchunks = btn_drawchunks.clone();
        let mnu_shapeselect = mnu_shapeselect.clone();
        let btn_step = btn_step.clone();
        let btn_clear = btn_clear.clone();
        let btn_mirror_shape = btn_mirror_shape.clone();
        let btn_rotate_shape = btn_rotate_shape.clone();
        let inp_update_intervall = inp_update_intervall.clone();
        let lbl_coords = lbl_coords.clone();

        wind.resize_callback(move |_, _, _, width, height| {
            //TODO remove some magic numbers
            canvas.borrow_mut().set_size(width, height);

            let padding = 5;
            let mut cur_x = width - padding - 100 - padding - 40 - padding - 40;
            let mut cur_y = padding;

            btn_clear.borrow_mut().set_pos(cur_x, cur_y);
            btn_clear.borrow_mut().set_size(40, 40);

            cur_x += padding + 40;

            btn_step.borrow_mut().set_pos(cur_x, cur_y);
            btn_step.borrow_mut().set_size(40, 40);

            cur_x += padding + 40;

            btn_stop_toggle.borrow_mut().set_pos(cur_x, cur_y);
            btn_stop_toggle.borrow_mut().set_size(100, 40);

            cur_y += padding + 40;

            btn_drawchunks.borrow_mut().set_pos(cur_x, cur_y);
            btn_drawchunks.borrow_mut().set_size(100, 20);

            cur_y += padding + 20;

            mnu_shapeselect.borrow_mut().set_pos(cur_x, cur_y);
            mnu_shapeselect.borrow_mut().set_size(100, 20);

            cur_y += padding + 20;

            btn_mirror_shape.borrow_mut().set_pos(cur_x, cur_y);
            btn_mirror_shape.borrow_mut().set_size(45, 40);

            cur_x += 2 * padding + 45;

            btn_rotate_shape.borrow_mut().set_pos(cur_x, cur_y);
            btn_rotate_shape.borrow_mut().set_size(45, 40);

            cur_x -= 2 * padding + 45;
            cur_y += padding + 40;

            inp_update_intervall.borrow_mut().set_pos(cur_x, cur_y);
            inp_update_intervall.borrow_mut().set_size(100, 20);

            lbl_coords.borrow_mut().set_pos(0, height);
            lbl_coords.borrow_mut().set_size(100, 0);
        });
    }

    {
        let canvas = canvas.clone();
        let btn_mirror_shape = btn_mirror_shape.clone();
        let btn_rotate_shape = btn_rotate_shape.clone();

        mnu_shapeselect
            .borrow_mut()
            .add("None", Shortcut::None, MenuFlag::Normal, move |_| {
                canvas.borrow_mut().set_curshape(None);
                btn_mirror_shape.borrow_mut().deactivate();
                btn_rotate_shape.borrow_mut().deactivate();
            });
        mnu_shapeselect.borrow_mut().set_value(0);
    }

    {
        let mut shapedir = fs::read_dir("./shapes/");

        match &mut shapedir {
            Ok(shapedir) => {
                let shapedir = return_first_err(shapedir);
                match shapedir {
                    Ok(shapedir) => {
                        for x in shapedir {
                            //TODO better error handling
                            if x.metadata().unwrap().is_file() {
                                let btn_mirror_shape = btn_mirror_shape.clone();
                                let btn_rotate_shape = btn_rotate_shape.clone();
                                let canvas = canvas.clone();

                                mnu_shapeselect.borrow_mut().add(
                                    x.file_name().into_string().unwrap().as_str(), //TODO better error handling
                                    Shortcut::None,
                                    MenuFlag::Normal,
                                    move |_| {
                                        let filepath = x.path();
                                        let mut curshape = parse_file(&filepath);
                                        if let Some(shape) = &mut curshape {
                                            //we have to mirror along the (0,0) -- (1,1) diagonal due to how we read the file
                                            mirror_diag(shape);
                                        }
                                        canvas.borrow_mut().set_curshape(curshape);
                                        btn_mirror_shape.borrow_mut().activate();
                                        btn_rotate_shape.borrow_mut().activate();
                                    },
                                );
                            }
                        }
                    }
                    Err(error) => println!("{error}"),
                }
            }
            Err(error) => println!("{error}"),
        }
    }

    {
        let canvas = canvas.clone();

        btn_clear.borrow_mut().set_callback(move |_| {
            canvas.borrow_mut().clear();
        });
    }

    {
        let canvas = canvas.clone();

        btn_step.borrow_mut().set_callback(move |_| {
            canvas.borrow_mut().update_threaded(10, 50);
        });
    }

    {
        let canvas = canvas.clone();
        let curshape = canvas.borrow().get_curshaperef();

        btn_mirror_shape.borrow_mut().set_callback(move |_| {
            let mut curshape = curshape.borrow_mut();
            if let Some(shape) = &mut *curshape {
                shape.reverse();
            }
        });
    }

    {
        let canvas: Rc<RefCell<Canvas>> = canvas.clone();

        btn_rotate_shape.borrow_mut().set_callback(move |_| {
            let curshape = canvas.borrow().get_curshaperef();
            let mut curshape = curshape.borrow_mut();

            if let Some(shape) = &mut *curshape {
                let mut rotshape = Vec::new();

                let vect_maxlen = shape.iter().fold(0, |x, y| std::cmp::max(x, y.len()));

                for i in 0..vect_maxlen {
                    let mut curcolumn = Vec::new();
                    let shapeiter = shape.iter();
                    for j in shapeiter {
                        curcolumn.push(*j.get(i).unwrap_or(&None));
                    }
                    rotshape.push(curcolumn);
                }
                rotshape.reverse();
                *shape = rotshape;
            }
        });
    }

    {
        let mut intervall = INITIALUPDATEINTERVALL;
        let mut timeouthandle = None;

        let canvas = canvas.clone();

        let hidewidgets: Vec<Rc<RefCell<dyn WidgetExt>>> = vec![
            btn_step.clone(),
            inp_update_intervall.clone(),
            mnu_shapeselect.clone(),
            btn_mirror_shape.clone(),
            btn_rotate_shape.clone(),
            btn_clear.clone(),
        ];

        let inp_update_intervall = inp_update_intervall.clone();

        btn_stop_toggle.borrow_mut().set_callback(move |handle| {
            if handle.value() {
                if let Some(timeouthandle) = timeouthandle {
                    remove_timeout3(timeouthandle);
                }

                canvas.borrow_mut().set_drawmode(true);
                inp_update_intervall
                    .borrow_mut()
                    .set_value(format! {"{intervall}"}.as_str());

                for widgetref in &hidewidgets {
                    widgetref.borrow_mut().show();
                }

                handle.set_label("Start");
            } else {
                let oldintervall = intervall;
                let newintervall = inp_update_intervall
                    .borrow()
                    .value()
                    .parse()
                    .unwrap_or(oldintervall);

                if 0.0 <= newintervall {
                    intervall = newintervall;
                }
                canvas.borrow_mut().set_drawmode(false);

                for widgetref in &hidewidgets {
                    widgetref.borrow_mut().hide();
                }

                handle.set_label("Stop");

                let canvas = canvas.clone();

                let update = move |handle| {
                    let start = std::time::Instant::now();
                    canvas.borrow_mut().update_threaded(10, 50);
                    app::repeat_timeout3(intervall - start.elapsed().as_secs_f64(), handle);
                };

                timeouthandle = Some(app::add_timeout3(intervall, update));
            }
        });
    }

    {
        let mut starttime_tick = std::time::Instant::now();

        let btn_stop_toggle = btn_stop_toggle.clone();
        let btn_step = btn_step.clone();
        let btn_mirror_shape = btn_mirror_shape.clone();
        let btn_rotate_shape = btn_rotate_shape.clone();

        let tick = move |handle| {
            let offset = canvas.borrow().offset();
            let xoffset = offset.0;
            let yoffset = offset.1;
            let linedist = canvas.borrow().linedist();

            canvas
                .borrow_mut()
                .redraw_canvas(btn_drawchunks.borrow().value());
            btn_stop_toggle.borrow_mut().redraw();
            btn_step.borrow_mut().redraw();
            btn_mirror_shape.borrow_mut().redraw();
            btn_rotate_shape.borrow_mut().redraw();

            let xmod = (app::event_x() + xoffset).rem_euclid(linedist);
            let ymod = (app::event_y() + yoffset).rem_euclid(linedist);

            let curcellmousepos = (
                ((app::event_x() + xoffset - xmod) / linedist),
                ((app::event_y() + yoffset - ymod) / linedist),
            );

            lbl_coords
                .borrow_mut()
                .set_label(format!("X: {} Y: {}", curcellmousepos.0, curcellmousepos.1).as_str());

            app::repeat_timeout3(TICKTIME - starttime_tick.elapsed().as_secs_f64(), handle);
            starttime_tick = std::time::Instant::now();
        };

        app::add_timeout3(TICKTIME, tick);
    }
    wind.show();
    app.run().unwrap();
}
