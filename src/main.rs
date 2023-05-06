#![windows_subsystem = "windows"]

use fltk::{
    app,
    app::remove_timeout3,
    button::{Button, CheckButton, ToggleButton},
    enums::{Event, FrameType, Shortcut},
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
    //rotshape.reverse();
    *vec = rotshape;
}

fn parse_file(file: &PathBuf) -> Option<Shape> {
    //Reads entire file into buffer, not a great idea for huge files
    let bytebuf: Vec<u8> = std::fs::read(file).expect("Todo file read error");
    let mut curshape = Vec::new();

    let bytebuflines = bytebuf
        .split(|b| *b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line));

    for line in bytebuflines {
        curshape.push(Vec::new());
        for b in line {
            if *b == b'0' {
                curshape.last_mut().unwrap().push(Some(false));
            } else if *b == b'1' {
                curshape.last_mut().unwrap().push(Some(true));
            } else {
                curshape.last_mut().unwrap().push(None);
            }
        }
    }
    Some(curshape)
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Game of Life");

    //let curshape = Rc::new(RefCell::new(None));
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

    //TODO perhaps add clear button?
    let btn_step = Button::default().with_label("Step");
    wind.add(&btn_step);

    let mut btn_stop_toggle = ToggleButton::default().with_label("Stop");
    btn_stop_toggle.set_id("ToggleBtn");
    btn_stop_toggle.set_value(true);
    btn_stop_toggle.set_shortcut(fltk::enums::Shortcut::Alt);
    wind.add(&btn_stop_toggle);

    //wind.add(&pck_updatefield);

    let btn_drawchunks = CheckButton::default().with_label("Draw chunks");
    wind.add(&btn_drawchunks);

    /*
    let mut grp_alwaysvisible = Group::default().with_pos(0, 0).with_size(WIDTH, HEIGHT);
    grp_alwaysvisible.add(&btn_stop_toggle);
    grp_alwaysvisible.add(&btn_drawchunks);
    grp_alwaysvisible.make_resizable(false);
    wind.add(&grp_alwaysvisible);
    */

    let mnu_shapeselect = Choice::default().with_label("Insert shape:");
    wind.add(&mnu_shapeselect);

    let mut btn_mirror_shape = Button::default().with_label("Flip");

    btn_mirror_shape.deactivate(); //TODO deactivating widget causes resize to not be respected
    wind.add(&btn_mirror_shape);

    let mut btn_rotate_shape = Button::default().with_label("Rotate");
    btn_rotate_shape.deactivate(); //TODO deactivating widget causes resize to not be respected
    wind.add(&btn_rotate_shape);

    let mut inp_update_intervall = FloatInput::default().with_label("Update intervall:");
    inp_update_intervall.set_value(format!("{}", INITIALUPDATEINTERVALL).as_str());
    wind.add(&inp_update_intervall);

    /*
    let mut grp_hidewidgets = Group::default().with_pos(WIDTH-145, 0).with_size(WIDTH, HEIGHT);
    grp_hidewidgets.add(&inp_update_intervall);
    grp_hidewidgets.add(&btn_step);
    grp_hidewidgets.add(&mnu_shapeselect);
    grp_hidewidgets.add(&pck_shapewidgets);
    grp_hidewidgets.make_resizable(false);
    wind.add(&grp_hidewidgets);
    */

    //let mut lbl_coords = TextDisplay::new(0,HEIGHT,100,0,"");
    let mut lbl_coords = TextDisplay::default();
    lbl_coords.set_frame(FrameType::NoBox); //<- i hate this
    wind.add(&lbl_coords);

    /*
    let mut pck_leftbarwidgets = Pack::default().with_pos(WIDTH-145, 0).with_size(145, HEIGHT).with_type(PackType::Vertical);
    pck_leftbarwidgets.set_spacing(5);
    pck_leftbarwidgets.make_resizable(false);
    pck_leftbarwidgets.add(&pck_updatefield);
    pck_leftbarwidgets.add(&btn_drawchunks);
    pck_leftbarwidgets.add(&mnu_shapeselect);
    pck_leftbarwidgets.add(&pck_shapewidgets);
    pck_leftbarwidgets.add(&inp_update_intervall);
    wind.add(&pck_leftbarwidgets);
    */

    let canvas = Rc::new(RefCell::new(canvas));
    let btn_stop_toggle = Rc::new(RefCell::new(btn_stop_toggle));
    let btn_drawchunks = Rc::new(RefCell::new(btn_drawchunks));
    let mnu_shapeselect = Rc::new(RefCell::new(mnu_shapeselect));
    let btn_step = Rc::new(RefCell::new(btn_step));
    let btn_mirror_shape = Rc::new(RefCell::new(btn_mirror_shape));
    let btn_rotate_shape = Rc::new(RefCell::new(btn_rotate_shape));
    //let pck_shapewidgets = Rc::new(RefCell::new(pck_shapewidgets));
    let inp_update_intervall = Rc::new(RefCell::new(inp_update_intervall));
    let lbl_coords = Rc::new(RefCell::new(lbl_coords));

    //let curshape = Rc::new(RefCell::new(None));

    {
        let canvas = canvas.clone();
        let btn_stop_toggle = btn_stop_toggle.clone();
        let btn_drawchunks = btn_drawchunks.clone();
        let mnu_shapeselect = mnu_shapeselect.clone();
        let btn_step = btn_step.clone();
        let btn_mirror_shape = btn_mirror_shape.clone();
        let btn_rotate_shape = btn_rotate_shape.clone();
        let inp_update_intervall = inp_update_intervall.clone();
        let lbl_coords = lbl_coords.clone();

        wind.resize_callback(move |_, _, _, width, height| {
            //TODO remove magic numbers
            canvas.borrow_mut().set_size(width, height);

            btn_stop_toggle.borrow_mut().set_pos(width - 100 - 5, 5);
            btn_stop_toggle.borrow_mut().set_size(100, 40);

            btn_drawchunks.borrow_mut().set_pos(width - 100 - 5, 45 + 5);
            btn_drawchunks.borrow_mut().set_size(100, 20);

            mnu_shapeselect
                .borrow_mut()
                .set_pos(width - 100 - 5, 70 + 5);
            mnu_shapeselect.borrow_mut().set_size(100, 20);

            btn_step.borrow_mut().set_pos(width - 145 - 5, 5);
            btn_step.borrow_mut().set_size(40, 40);

            btn_mirror_shape
                .borrow_mut()
                .set_pos(width - 100 - 5, 95 + 5);
            btn_mirror_shape.borrow_mut().set_size(45, 40);

            btn_rotate_shape
                .borrow_mut()
                .set_pos(width - 45 - 5, 95 + 5);
            btn_rotate_shape.borrow_mut().set_size(45, 40);

            inp_update_intervall
                .borrow_mut()
                .set_pos(width - 100 - 5, 140 + 5);
            inp_update_intervall.borrow_mut().set_size(100, 20);

            lbl_coords.borrow_mut().set_pos(0, height);
            lbl_coords.borrow_mut().set_size(100, 0);

            //pck_leftbarwidgets.set_pos(width-145, 0);
            //pck_leftbarwidgets.set_size(145,height);
            //lbl_coords.borrow_mut().set_pos(0, height);
            //btn_stop_toggle.borrow_mut().set_pos(width-100,0);
            //btn_drawchunks.borrow_mut().set_pos(btn_drawchunks.borrow().x(), y)
            //grp_hidewidgets.borrow_mut().set_size(width, height);
        });
    }

    {
        let canvas = canvas.clone();
        let btn_mirror_shape = btn_mirror_shape.clone();
        let btn_rotate_shape = btn_rotate_shape.clone();
        //let curshape = curshape.clone();

        mnu_shapeselect
            .borrow_mut()
            .add("None", Shortcut::None, MenuFlag::Normal, move |_| {
                canvas.borrow_mut().set_curshape(None);
                btn_mirror_shape.borrow_mut().deactivate();
                btn_rotate_shape.borrow_mut().deactivate();
                println!("None")
            });
        mnu_shapeselect.borrow_mut().set_value(0);
    }

    {
        let shapedir = fs::read_dir("./shapes/");

        match shapedir {
            Ok(shapedir) => {
                let shapedir = shapedir.into_iter().map(|x| x.unwrap());
                for x in shapedir {
                    if x.metadata().unwrap().is_file() {
                        //let curshaperef = curshape.clone();
                        let btn_mirror_shape = btn_mirror_shape.clone();
                        let btn_rotate_shape = btn_rotate_shape.clone();
                        let canvas = canvas.clone();

                        mnu_shapeselect.borrow_mut().add(
                            x.file_name().into_string().unwrap().as_str(),
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
                                println!("{}", filepath.to_str().unwrap());
                            },
                        );
                    }
                }
            }
            Err(error) => println!("{}", error),
        }
    }

    {
        let canvas = canvas.clone();

        btn_step.borrow_mut().set_callback(move |_| {
            let start = std::time::Instant::now();
            canvas.borrow_mut().update();
            println!("Step took {} ms", start.elapsed().as_millis());
        });
    }

    {
        let canvas = canvas.clone();
        let curshape = canvas.borrow().get_curshaperef();

        btn_mirror_shape.borrow_mut().handle(move |_, ev| match ev {
            Event::Push => {
                let mut curshape = curshape.borrow_mut();
                if let Some(shape) = &mut *curshape {
                    shape.reverse();
                }
                true
            }
            Event::Resize => {
                println!("btn_rotate_shape resized");
                true
            }
            _ => false,
        });
    }

    {
        let canvas = canvas.clone();
        //let curshape = curshape.clone();

        btn_rotate_shape.borrow_mut().handle(move |_, ev| match ev {
            Event::Push => {
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
                true
            }
            Event::Deactivate => {
                println!("btn_rotate_shape resized");
                true
            }
            _ => false,
        });
    }

    {
        let mut intervall = INITIALUPDATEINTERVALL;
        let mut timeouthandle = app::add_timeout3(core::f64::MAX, |_| ()); //<- i despise this. creates dummy timer just to fill timeouthandle

        let canvas = canvas.clone();

        let hidewidgets: Vec<Rc<RefCell<dyn WidgetExt>>> = vec![
            btn_step.clone(),
            inp_update_intervall.clone(),
            mnu_shapeselect.clone(),
            btn_mirror_shape.clone(),
            btn_rotate_shape.clone(),
        ];

        let inp_update_intervall = inp_update_intervall.clone();

        btn_stop_toggle.borrow_mut().set_callback(move |handle| {
            println!("Toggle Btn Pressed");
            if handle.value() {
                remove_timeout3(timeouthandle);
                //println!("timer destroyed");

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
                //------------------------------------
                //let btn_stop_toggleref1 = btn_stop_toggleref.clone();
                //let inp_update_intervallref2 = inp_update_intervallref.clone();
                let canvas = canvas.clone();
                //let intervall = intervall.clone();

                let update = move |handle| {
                    let start = std::time::Instant::now();
                    canvas.borrow_mut().update_threaded(10, 50);

                    let secs = start.elapsed().as_secs_f64();
                    println!(
                        "elapsed time is {} s, setting timout {} s\n{} alive chunks",
                        secs,
                        intervall - secs,
                        canvas.borrow().len()
                    );

                    app::repeat_timeout3(intervall - start.elapsed().as_secs_f64(), handle);
                };

                timeouthandle = app::add_timeout3(intervall, update);
                //println!("timer started for the first time with timeout {}", *intervall.borrow() );

                //------------------------------------
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

            //TODO find better way to redraw stuff, use damage values to determine what needs to be redrawn
            canvas
                .borrow_mut()
                .redraw_canvas(btn_drawchunks.borrow().value()); //TODO this can take long, i could collect the time it takes or smth and substract it from the update intervall
            btn_stop_toggle.borrow_mut().redraw();
            btn_step.borrow_mut().redraw();
            btn_mirror_shape.borrow_mut().redraw();
            btn_rotate_shape.borrow_mut().redraw();
            //println!("updated canvas");
            //println!("redrew canvas in {} ms", start.elapsed().as_millis());

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
    //wind.end();
    wind.show();
    app.run().unwrap();
}
