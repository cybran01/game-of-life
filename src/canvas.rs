use std::cell::RefCell;
use std::rc::Rc;

use fltk::{
    app,
    app::MouseButton,
    draw::{draw_rect, draw_rect_fill, set_draw_color, set_line_style, LineStyle},
    enums::{Color, Event, FrameType},
    frame::Frame,
    prelude::*,
    surface::ImageSurface,
};

use crate::field::{Field, Shape};
fltk::widget_extends!(Canvas, Frame, frame);
pub struct Canvas {
    chunksize: usize,
    frame: Frame,
    surf: Rc<RefCell<ImageSurface>>,
    field: Rc<RefCell<Field>>,
    drawmode: Rc<RefCell<bool>>,
    xoffsetref: Rc<RefCell<i32>>,
    yoffsetref: Rc<RefCell<i32>>,
    linedistref: Rc<RefCell<i32>>,
    shaperef: Rc<RefCell<Option<Shape>>>,
}

impl Canvas {
    pub fn new(
        w: i32,
        h: i32,
        chunksize: usize,
        xoffset: i32,
        yoffset: i32,
        linedist: i32,
    ) -> Self {
        let mut frame = Frame::default().with_size(w, h).center_of_parent();
        let field = Rc::new(RefCell::new(Field::new(chunksize)));
        let drawmode = Rc::new(RefCell::new(true));
        let shaperef = Rc::new(RefCell::new(None));

        frame.set_color(Color::White);
        frame.set_frame(FrameType::DownBox);

        let xoffsetref = Rc::new(RefCell::new(xoffset));
        let yoffsetref = Rc::new(RefCell::new(yoffset));
        let linedistref = Rc::new(RefCell::new(linedist));

        let surf = ImageSurface::new(frame.width(), frame.height(), false);
        let surf = Rc::from(RefCell::from(surf));

        frame.draw({
            let surf = surf.clone();
            move |f| {
                let surf = surf.borrow();
                let mut img = surf.image().unwrap();
                img.draw(f.x(), f.y(), f.w(), f.h());
            }
        });

        frame.handle({
            let mut lastclickedcoords = (0, 0);
            let mut lastsetfieldcoords = (0, 0);

            //let surf = surf.clone();
            let field = field.clone();
            let xoffsetref = xoffsetref.clone();
            let yoffsetref = yoffsetref.clone();
            let linedistref = linedistref.clone();
            let drawmode = drawmode.clone();
            let shaperef = shaperef.clone();

            //let shaperef = shaperef.clone();

            move |_, ev| {
                //let mut surf = surf.borrow();
                let mut field = field.borrow_mut();

                match ev {
                    Event::Push => {
                        let coords = app::event_coords();
                        lastclickedcoords = coords;

                        if *drawmode.borrow()
                            && app::event_mouse_button() == app::MouseButton::Right
                        {
                            let xoffset = *xoffsetref.borrow();
                            let yoffset = *yoffsetref.borrow();
                            let linedist = *linedistref.borrow();

                            let xmod = (coords.0 + xoffset).rem_euclid(linedist);
                            let ymod = (coords.1 + yoffset).rem_euclid(linedist);

                            let fieldcoords = (
                                ((coords.0 + xoffset - xmod) / linedist) as isize,
                                ((coords.1 + yoffset - ymod) / linedist) as isize,
                            );
                            let curshape = &*shaperef.borrow();
                            match curshape {
                                Some(shape) => {
                                    field.set_shape_at(fieldcoords, shape);
                                }
                                None => {
                                    let curval = field.get_cell(fieldcoords.0, fieldcoords.1);
                                    field.set_cell(fieldcoords, !curval);
                                    lastsetfieldcoords = fieldcoords;
                                }
                            }
                        }
                        true
                    }
                    Event::Drag => {
                        if app::event_mouse_button() == MouseButton::Left {
                            //println!("Drag left");

                            let coords = app::event_coords();

                            let newxoffset =
                                *xoffsetref.borrow() - (coords.0 - lastclickedcoords.0);
                            let newyoffset =
                                *yoffsetref.borrow() - (coords.1 - lastclickedcoords.1);

                            xoffsetref.replace(newxoffset);
                            yoffsetref.replace(newyoffset);

                            lastclickedcoords = coords;
                            true
                        } else if app::event_mouse_button() == MouseButton::Right
                            && shaperef.borrow().is_none()
                        {
                            //println!("Drag right");
                            if *drawmode.borrow()
                                && app::event_mouse_button() == app::MouseButton::Right
                            {
                                let coords = app::event_coords();

                                let xoffset = *xoffsetref.borrow();
                                let yoffset = *yoffsetref.borrow();
                                let linedist = *linedistref.borrow();

                                let xmod = (coords.0 + xoffset).rem_euclid(linedist);
                                let ymod = (coords.1 + yoffset).rem_euclid(linedist);

                                let fieldcoords = (
                                    ((coords.0 + xoffset - xmod) / linedist) as isize,
                                    ((coords.1 + yoffset - ymod) / linedist) as isize,
                                );
                                let curval = field.get_cell(fieldcoords.0, fieldcoords.1);

                                if fieldcoords != lastsetfieldcoords {
                                    field.set_cell(fieldcoords, !curval);
                                    lastsetfieldcoords = fieldcoords;
                                }
                            }
                            true
                        } else {
                            false
                        }
                    }
                    Event::MouseWheel => {
                        //println!("MouseWheel");
                        let coords: (i32, i32) = app::event_coords();

                        let xoffset = *xoffsetref.borrow();
                        let yoffset = *yoffsetref.borrow();
                        let linedist = *linedistref.borrow();

                        match app::event_dy() {
                            app::MouseWheel::Up => {
                                if linedist > 2 {
                                    *xoffsetref.borrow_mut() -= (coords.0 + xoffset) / linedist;
                                    *yoffsetref.borrow_mut() -= (coords.1 + yoffset) / linedist;
                                    (*linedistref.borrow_mut()) -= 1;
                                }
                            }
                            app::MouseWheel::Down => {
                                *xoffsetref.borrow_mut() += (coords.0 + xoffset) / linedist;
                                *yoffsetref.borrow_mut() += (coords.1 + yoffset) / linedist;
                                (*linedistref.borrow_mut()) += 1;
                            }
                            _ => (),
                        }
                        true
                    }
                    _ => false,
                }
            }
        });
        Self {
            frame,
            surf,
            chunksize,
            field,
            drawmode,
            xoffsetref,
            yoffsetref,
            linedistref,
            shaperef,
        }
    }

    pub fn redraw_canvas(&mut self, drawchunks: bool) {
        let xoffset = *self.xoffsetref.borrow();
        let yoffset = *self.yoffsetref.borrow();
        let linedist = *self.linedistref.borrow();

        let xmod = xoffset.rem_euclid(linedist);
        let ymod = yoffset.rem_euclid(linedist);

        ImageSurface::push_current(&self.surf.borrow_mut());
        draw_rect_fill(0, 0, self.w(), self.h(), Color::White);

        set_draw_color(Color::Black);

        for xcoord in (linedist - xmod..=self.w()).step_by(linedist as usize) {
            fltk::draw::draw_line(xcoord, 0, xcoord, self.h());
        }
        for ycoord in (linedist - ymod..=self.h()).step_by(linedist as usize) {
            fltk::draw::draw_line(0, ycoord, self.w(), ycoord);
        }

        for xcoord in (-xmod..=self.w()).step_by(linedist as usize) {
            for ycoord in (-ymod..=self.h()).step_by(linedist as usize) {
                if self.field.borrow().get_cell(
                    ((xcoord + xoffset) / linedist) as isize,
                    ((ycoord + yoffset) / linedist) as isize,
                ) {
                    draw_rect_fill(xcoord, ycoord, linedist, linedist, Color::Black);
                }
            }
        }

        if drawchunks {
            let xoffset = *self.xoffsetref.borrow();
            let yoffset = *self.yoffsetref.borrow();
            let linedist = *self.linedistref.borrow() * self.chunksize as i32; //we treat chunks as cells with size linedist*DIM when drawing

            let xmod = xoffset.rem_euclid(linedist);
            let ymod = yoffset.rem_euclid(linedist);

            set_draw_color(Color::Red);
            set_line_style(LineStyle::Solid, 3);
            let filter = |(x, y): &&(isize, isize)| {
                (*x as i32) * linedist - xoffset >= -xmod
                    && (*x as i32) * linedist - xoffset <= self.w()
                    && (*y as i32) * linedist - yoffset >= -ymod
                    && (*y as i32) * linedist - yoffset <= self.h()
            };
            for (x, y) in self.field.borrow().vec.keys().filter(filter) {
                draw_rect(
                    (*x as i32) * linedist - xoffset,
                    (*y as i32) * linedist - yoffset,
                    linedist,
                    linedist,
                );
            }
            set_line_style(LineStyle::Solid, 0);
        }

        ImageSurface::pop_current();
        self.frame.redraw();
    }

    pub fn update(&mut self) {
        self.field.borrow_mut().update();
    }

    pub fn update_threaded(&mut self, chunks_per_thread: usize, max_threads: usize) {
        self.field
            .borrow_mut()
            .update_threaded(chunks_per_thread, max_threads);
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.frame.set_size(width, height);
        *self.surf.borrow_mut() = ImageSurface::new(width, height, false)
    }

    pub fn set_drawmode(&mut self, val: bool) {
        *self.drawmode.borrow_mut() = val;
    }

    pub fn drawmode(&self) -> bool {
        *self.drawmode.borrow()
    }

    pub fn set_offset(&mut self, val: (i32, i32)) {
        *self.xoffsetref.borrow_mut() = val.0;
        *self.yoffsetref.borrow_mut() = val.1;
    }

    pub fn offset(&self) -> (i32, i32) {
        (*self.xoffsetref.borrow(), *self.yoffsetref.borrow())
    }

    pub fn set_linedist(&mut self, val: i32) {
        *self.linedistref.borrow_mut() = val;
    }

    pub fn linedist(&self) -> i32 {
        *self.linedistref.borrow()
    }

    pub fn set_curshape(&mut self, val: Option<Shape>) {
        *self.shaperef.borrow_mut() = val;
    }

    pub fn get_curshaperef(&self) -> Rc<RefCell<Option<Shape>>> {
        self.shaperef.clone()
    }

    pub fn len(&self) -> usize {
        self.field.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.field.borrow().is_empty()
    }
}
