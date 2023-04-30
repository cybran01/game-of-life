const DIM:usize = 8;

use std::collections::HashMap;
use std::sync::Arc;
use std::collections::hash_map::{Keys, IntoKeys};
use std::thread::Thread;
use std::{vec, isize, fs};
use std::{rc::Rc};
use std::io;

pub trait CellsGettable {
    fn get_cell(&self,x:usize,y:usize) -> bool;
}
#[derive(Clone)]
struct Square {
    cell: [[bool;DIM];DIM],
    alive_cells: i32
}
#[derive(Clone)]
struct Field {
    vec:std::collections::HashMap<(isize,isize),Square>
}

enum Direction {
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast
}

impl Direction {
    fn shift(coords:(isize,isize), dir:&Direction) -> (isize,isize){
        match dir {Direction::North => (coords.0,coords.1+1),
            Direction::South => (coords.0,coords.1-1),
            Direction::West => (coords.0-1,coords.1),
            Direction::East => (coords.0+1,coords.1),
            Direction::NorthWest => (coords.0-1,coords.1+1),
            Direction::NorthEast => (coords.0+1,coords.1+1),
            Direction::SouthWest =>(coords.0-1,coords.1-1),
            Direction::SouthEast => (coords.0+1,coords.1-1)
        }
    }
    fn iter() -> std::slice::Iter<'static, Direction> {
        use self::Direction::*;
        static DIRECTIONS: [Direction; 8] = [North, South, East, West, NorthWest, NorthEast,SouthWest, SouthEast];
        DIRECTIONS.iter()
    }
}

impl CellsGettable for Square {
    fn get_cell(&self,x:usize,y:usize) -> bool {
        assert!((0..DIM).contains(&x));
        assert!((0..DIM).contains(&y));
        self.cell[x][y]
    }
}

impl Square {
    fn new () -> Self { //initializes a Square with all cells dead
        let cells:[[bool;DIM];DIM] = [[false;DIM];DIM];
        Self{cell:cells,alive_cells:0}
    }
    fn get_cell(&self,x:usize,y:usize) -> bool {
        assert!((0..DIM).contains(&x));
        assert!((0..DIM).contains(&y));

        self.cell[x][y]
    }
    fn set_cell(&mut self,x:usize,y:usize,v:bool) {        
        assert!((0..DIM).contains(&x));
        assert!((0..DIM).contains(&y));

        if self.cell[x][y] && !v{
            self.alive_cells-=1;
        }
        else if !self.cell[x][y] && v {
            self.alive_cells +=1;
        }
        self.cell[x][y] = v;
    }
}
impl Field {
    fn new () -> Self {
        let vec = std::collections::HashMap::new();
        Self {vec}
    }
    fn get_cell(&self,x:isize,y:isize) -> bool {
        let dim = isize::try_from(DIM).unwrap();

        let coord_in_square = (usize::try_from(x.rem_euclid(DIM.try_into().unwrap())).unwrap(),usize::try_from(y.rem_euclid(DIM.try_into().unwrap())).unwrap());
        let square_coord = ((x-isize::try_from(coord_in_square.0).unwrap())/dim, (y-isize::try_from(coord_in_square.1).unwrap())/dim);
        
        match self.vec.get(&square_coord) {
            Some(square) => square.get_cell(coord_in_square.0,coord_in_square.1),
            None => false
        }
    }
    
    fn set_cell(&mut self, coords:(isize,isize), val:bool) {
        let localcoords = (coords.0.rem_euclid(DIM as isize),coords.1.rem_euclid(DIM as isize));
        let squarecoords = ((coords.0-localcoords.0)/DIM as isize,(coords.1-localcoords.1)/DIM as isize);

        match self.vec.get_mut(&squarecoords) {
            Some(cursquare) => {
                cursquare.set_cell(localcoords.0 as usize, localcoords.1 as usize, val);
                if cursquare.alive_cells == 0 {
                    self.vec.remove(&squarecoords);
                }
            },
            None => {
                if val {
                    let mut cursquare = Square::new();
                    cursquare.set_cell(localcoords.0 as usize, localcoords.1 as usize, true);
                    self.vec.insert(squarecoords, cursquare);
                }}
        }
    }

    fn set_shape_at(&mut self, coords:(isize,isize), shape:Vec<Vec<Option<bool>>>) { //takes vector of columns
        for x in 0..shape.len() {
            for y in 0..shape[x].len() {
                if let Some(val)= shape[x][y] {
                    self.set_cell((coords.0+x as isize,coords.1+y as isize), val);
                }
            }
        }
    }

    fn eval_cells_alive_on_boundary(&self, x:isize, y:isize) -> i32 { //TODO maybe do this via maxdist of x,y values
        let mut counter = 0;

        for dir in Direction::iter() {
            let coords = Direction::shift((x,y), dir);
            if self.get_cell(coords.0, coords.1) {
                counter+=1;
            }
        }
        return counter;
    }

    fn update_chunk(&self, coords:(isize,isize)) -> Option<Square> {
        let cursquare = self.vec.get(&coords);
        let mut square;
        let mut checkonlyboundary = false;

        match cursquare {
            Some(squareref) => square = squareref.clone(),
            None => {
                square = Square{cell:[[false;DIM];DIM],alive_cells:0};
                checkonlyboundary = true;
            }
        }
        for x in 0..DIM {
            for y in 0..DIM {
                if !checkonlyboundary || x == 0 || y == 0 || x == DIM-1 || y == DIM-1 {
                    let curcoord = (coords.0*isize::try_from(DIM).unwrap()+isize::try_from(x).unwrap(),coords.1*isize::try_from(DIM).unwrap()+isize::try_from(y).unwrap());

                    let cells_alive_on_boundary = self.eval_cells_alive_on_boundary(curcoord.0, curcoord.1);
                    let curcell = square.get_cell(x, y);

                    if !curcell && cells_alive_on_boundary == 3 { //a dead cell with 3 neighbors becomes alive
                        square.set_cell(x,y,true);
                    }
                    else if curcell && !(cells_alive_on_boundary == 2 || cells_alive_on_boundary == 3) { //an alive cell that does not have 2 or 3 alive neighbors, dies 
                        square.set_cell(x,y,false);
                    } 
                }
            }
        }
        if square.alive_cells == 0 {
            return None;
        }
        else {
            return Some(square);
        }
    }

    fn insert_valid_only(key:(isize,isize), elem:Option<Square>, hs:&mut std::collections::HashMap<(isize,isize),Square>) { //TODO check if try_insert is finally out of nightly
        if let Some(_) = elem {
                if !hs.contains_key(&key) {
                    hs.insert(key, elem.unwrap());
                }
        }
    }

    fn update (&mut self) {
        let allkeys = self.vec.clone().into_keys();
        self.vec = self.update_keys(allkeys);
    }

    fn update_keys<T:IntoIterator<Item = (isize,isize)>> (&self, keys:T) -> std::collections::HashMap<(isize,isize),Square> {
        let mut hs = std::collections::HashMap::new();

        for (x,y) in keys {
            let newchunk = self.update_chunk((x,y));
            Self::insert_valid_only((x,y),newchunk,&mut hs);

            for dir in Direction::iter() {
                let coords = Direction::shift((x,y), dir);
                let newchunk = self.update_chunk(coords);
                Self::insert_valid_only(coords,newchunk,&mut hs);
            }
            //The following is not possible anymore when using parallel code
            //self.vec.remove_entry(&(x,y));
        }
        hs
    }
    fn update_threaded (&mut self) {
        let allkeys = self.vec.clone().into_keys();
        
        let threadnum = std::cmp::max(allkeys.size_hint().0/10,1);
        println!("At least {} chunks, hence {} thread(s) to spawn", allkeys.size_hint().0, std::cmp::max(allkeys.size_hint().0/10,1));

        let mut splitkeysvec = vec![Vec::new();threadnum];
        for (i,x) in allkeys.enumerate() {
            splitkeysvec.get_mut(i%threadnum).unwrap().push(x);
        }

        //let mut hs:HashMap<(isize,isize),Square> = std::collections::HashMap::new();
        let mut res = Vec::new(); 

        std::thread::scope(|s| {
            let mut handlevec = Vec::new();

            for x in splitkeysvec {
                handlevec.push(s.spawn(|| self.update_keys(x)));
            }
            res = handlevec.into_iter().map(|x| x.join().unwrap()).collect();
        });

        self.vec.clear();
        for x in res {
            self.vec.extend(x);
        }
        
    }
}

fn print (f:&Field) {
    let dim = isize::try_from(DIM).unwrap();

    for x in -1*dim..2*dim {
        for y in -1*dim..2*dim {
            print!("{}", i32::try_from(f.get_cell(x, y)).unwrap());
        }
        println!();
    }
}

use std::cell::{RefCell, Ref};
struct Canvas {
    frame: Frame,
    surf: Rc<RefCell<ImageSurface>>,
    field:Rc<RefCell<Field>>,
    xoffsetref:Rc<RefCell<i32>>,
    yoffsetref:Rc<RefCell<i32>>,
    linedistref:Rc<RefCell<i32>>,
    drawmoderef:Rc<RefCell<bool>>
}

impl Canvas {
    pub fn new(w: i32, h: i32, field: Rc<RefCell<Field>>,xoffset:i32,yoffset:i32, linedist:i32) -> Self {
        let mut frame = Frame::default().with_size(w, h).center_of_parent();
        let drawmoderef = Rc::new(RefCell::new(true));
        
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
            let mut lastclickedcoords = (0,0);
            let mut lastsetfieldcoords = (0,0);

            let surf = surf.clone();
            let field = field.clone();
            let xoffsetref = xoffsetref.clone();
            let yoffsetref = yoffsetref.clone();
            let linedistref = linedistref.clone();
            let drawmoderef = drawmoderef.clone();

            move |f, ev| {
                let mut surf = surf.borrow();
                let mut field = field.borrow_mut(); 

                match ev {
                    Event::Push => { //TODO the first part is probably only necessary when left clicked
                        //println!("Push");
                        let coords = app::event_coords();

                        //println!("{}+{}",coords.0,*xoffsetref.borrow());
                        //println!("{}+{}",coords.1,*yoffsetref.borrow());

                        lastclickedcoords = coords;

                        if *drawmoderef.borrow() && app::event_mouse_button() == app::MouseButton::Right {
                            let xoffset = *xoffsetref.borrow();
                            let yoffset = *yoffsetref.borrow();
                            let linedist = *linedistref.borrow();

                            let xmod = (coords.0+xoffset).rem_euclid(linedist);
                            let ymod = (coords.1+yoffset).rem_euclid(linedist);

                            let fieldcoords = (((coords.0+xoffset-xmod)/linedist) as isize,((coords.1+yoffset-ymod)/linedist) as isize);
                            let curval = field.get_cell(fieldcoords.0, fieldcoords.1);

                            field.set_cell(fieldcoords, !curval);
                            lastsetfieldcoords = fieldcoords;
                        }
                        true
                    }
                    Event::Drag => {
                        if app::event_mouse_button() == MouseButton::Left {
                            //println!("Drag left");

                            let coords = app::event_coords();

                            let newxoffset = *xoffsetref.borrow()-(coords.0-lastclickedcoords.0);
                            let newyoffset = *yoffsetref.borrow()-(coords.1-lastclickedcoords.1);
    
                            xoffsetref.replace(newxoffset);
                            yoffsetref.replace(newyoffset);
    
                            lastclickedcoords = coords;
                            true
                        }
                        else if app::event_mouse_button() == MouseButton::Right { 
                            //println!("Drag right");

                            let coords = app::event_coords();
      
                            //lastclickedcoords = coords; //probably not necessary 

                            //println!("{}+{}",coords.0,*xoffsetref.borrow());
                            //println!("{}+{}",coords.1,*yoffsetref.borrow());

                            if *drawmoderef.borrow() && app::event_mouse_button() == app::MouseButton::Right {
                                let xoffset = *xoffsetref.borrow();
                                let yoffset = *yoffsetref.borrow();
                                let linedist = *linedistref.borrow();

                                let xmod = (coords.0+xoffset).rem_euclid(linedist);
                                let ymod = (coords.1+yoffset).rem_euclid(linedist);

                                let fieldcoords = (((coords.0+xoffset-xmod)/linedist) as isize,((coords.1+yoffset-ymod)/linedist) as isize);
                                let curval = field.get_cell(fieldcoords.0, fieldcoords.1);

                                if fieldcoords != lastsetfieldcoords {
                                    field.set_cell(fieldcoords, !curval);
                                    lastsetfieldcoords = fieldcoords;
                                }
                            }
                            true
                        }
                        else {
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
                                    *xoffsetref.borrow_mut()-= (coords.0+xoffset)/linedist;
                                    *yoffsetref.borrow_mut()-= (coords.1+yoffset)/linedist;
                                    (*linedistref.borrow_mut())-=1;
                                }}, 
                            app::MouseWheel::Down => {
                                *xoffsetref.borrow_mut()+= (coords.0+xoffset)/linedist;
                                *yoffsetref.borrow_mut()+= (coords.1+yoffset)/linedist;
                                (*linedistref.borrow_mut())+=1;
                            }, 
                            _ => ()
                        }
                        true
                    },
                    _ => false,
                }
            }
        });
        let mut newobj = Self {frame, surf , field, xoffsetref, yoffsetref, linedistref, drawmoderef};
        newobj.redraw_canvas(false);
        newobj
    }

    fn redraw_canvas(&mut self, drawchunks:bool) {
        let xoffset = *self.xoffsetref.borrow();
        let yoffset = *self.yoffsetref.borrow();
        let linedist = *self.linedistref.borrow();

        let xmod = xoffset.rem_euclid(linedist);
        let ymod = yoffset.rem_euclid(linedist);
        
        ImageSurface::push_current(&self.surf.borrow_mut());
        draw_rect_fill(0, 0, WIDTH, HEIGHT, Color::White);
        
        set_draw_color(Color::Black);

        for xcoord in (linedist-xmod..=WIDTH).step_by(linedist as usize) {
            fltk::draw::draw_line(xcoord, 0, xcoord, HEIGHT);
        }
        for ycoord in (linedist-ymod..=HEIGHT).step_by(linedist as usize) {
            fltk::draw::draw_line(0, ycoord, WIDTH, ycoord);
        }

        for xcoord in (-xmod..=WIDTH).step_by(linedist as usize) {
            for ycoord in (-ymod..=HEIGHT).step_by(linedist as usize) {
                if self.field.borrow().get_cell(((xcoord+xoffset)/linedist) as isize, ((ycoord+yoffset)/linedist) as isize) {
                    draw_rect_fill(xcoord, ycoord,linedist, linedist,Color::Black);
                }
            }
        }

        if drawchunks {
            let xoffset = *self.xoffsetref.borrow();
            let yoffset = *self.yoffsetref.borrow();
            let linedist = *self.linedistref.borrow()*DIM as i32; //we treat chunks as cells with size linedist*DIM when drawing

            let xmod = xoffset.rem_euclid(linedist);
            let ymod = yoffset.rem_euclid(linedist);

            set_draw_color(Color::Red);
            set_line_style(LineStyle::Solid, 3);
            let filter = |(x,y):&&(isize,isize)| {
                if (*x as i32)*linedist-xoffset>=-xmod && (*x as i32)*linedist-xoffset<=WIDTH && (*y as i32)*linedist-yoffset>=-ymod && (*y as i32)*linedist-yoffset<=HEIGHT {
                    //println!("Took {},{}",*x,*y);
                    true
                }
                else {
                    false
                }
            };
            for (x,y) in self.field.borrow().vec.keys().filter(filter) {
                draw_rect((*x as i32)*linedist-xoffset, (*y as i32)*linedist-yoffset, linedist, linedist);
            }
            set_line_style(LineStyle::Solid, 0);
        }

        ImageSurface::pop_current();
        self.frame.redraw();
    } 
}

const WIDTH: i32 = 800*2;
const HEIGHT: i32 = 600*2;
const TICKTIME: f64 = 0.05; 
const STARTLINEDIST: i32 = 30;
const INITIALUPDATEINTERVALL: f64 = 0.1; 
const XSTARTOFFSET: i32 = 0;
const YSTARTOFFSET: i32 = 0;
const DRAWCHUNKS: bool = true;

fltk::widget_extends!(Canvas, Frame, frame);

use fltk::app::{remove_timeout3, TimeoutHandle, handle, MouseButton};
use fltk::button;
use fltk::draw::draw_rect;
use fltk::enums::{Shortcut, CallbackTrigger};
use fltk::menu::MenuFlag;
use fltk::{
    app,
    draw::{draw_line, draw_point, draw_rect_fill, set_draw_color, set_line_style, LineStyle},
    enums::{Color, Event, FrameType},
    frame::Frame,
    prelude::*,
    surface::ImageSurface,
    window::Window, button::ToggleButton, text::TextDisplay, input::FloatInput, button::Button, button::CheckButton, menu::Choice
};

//helper function for rotating double vecs
fn rot<T:Copy> (vec:&Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut res = Vec::new();
    for i in 0..vec.len() {
        let mut tmp = Vec::new();
        for j in 0..vec[i].len() {
            tmp.push(vec[j][i]);
        }
        res.push(tmp);
    }
    res
}

fn to_opt<T> (vec:Vec<T>) -> Vec<Option<T>> {
    vec.into_iter().map(|x|Some(x)).collect()
}

fn to_opt2<T> (vec:Vec<Vec<T>>) -> Vec<Vec<Option<T>>> {
    vec.into_iter().map(|x|to_opt(x)).collect()
}

fn main() {
    //-----------------------------------------------------------------------------------------
    let glider =vec![vec![false,true,false],
                                    vec![false,false,true],
                                    vec![true,true,true]];

    let mut field = Field::new();
    field.set_shape_at((0,0),to_opt2(rot(&glider))); 
    
    //-----------------------------------------------------------------------------------------

    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Game of Life");
    
    let f = RefCell::new(field);
    let canvas = Canvas::new(WIDTH, HEIGHT, f.into(), XSTARTOFFSET, YSTARTOFFSET, STARTLINEDIST);
    wind.add(&canvas.frame);
    let canvasref = Rc::new(RefCell::new(canvas));

    let mut btn_stop_toggle = ToggleButton::default().with_label("Stop").with_size(100, 40).with_pos(WIDTH-100,0);
    btn_stop_toggle.set_id("ToggleBtn");
    btn_stop_toggle.set_value(true);
    btn_stop_toggle.set_shortcut(fltk::enums::Shortcut::Alt); 

    let mut btn_drawchunks = CheckButton::default().with_size(100,20).with_label("Draw chunks").below_of(&btn_stop_toggle, 5);
    wind.add(&btn_drawchunks);

    let mut mnu_shapeselect = Choice::default().with_size(100,20).below_of(&btn_drawchunks, 5).with_label("Insert shape:");
    let shapedir = fs::read_dir("./shapes/").unwrap().map(|x| x.unwrap()); //TODO this can fail silenty e.g. when permissions are missing 
    let mut menue = String::from("None");
    for x in shapedir {
        if x.metadata().unwrap().is_file() {
            let mut nextitem = String::from("|");
            nextitem.push_str(x.file_name().into_string().unwrap().as_str());
            menue.push_str(nextitem.as_str());
        }
    }
    println!("{}",menue);
    mnu_shapeselect.clear();
    mnu_shapeselect.add_choice(&menue);
    //handle.set_value(0);

    let mut last_selection = 0;
    mnu_shapeselect.set_value(0);

    mnu_shapeselect.set_callback(move |handle| {
        if handle.value() == -1 {
            handle.set_value(last_selection);
        }
        if last_selection != handle.value() {
            println!("choose nr {}", handle.value());
        }
        last_selection = handle.value();       
    });

    mnu_shapeselect.handle(move |handle, event| {
        match event {
            Event::Leave => {
                handle.do_callback();
                true
            },
            _ => false
        }
    });
    wind.add(&mnu_shapeselect);

    let mut btn_step = Button::default().with_label("Step").with_size(40, 40).left_of(&btn_stop_toggle, 5);
    let canvasref3 = canvasref.clone();
    btn_step.set_callback(move |_| {
        let start = std::time::Instant::now();
        canvasref3.borrow_mut().field.borrow_mut().update();
        println!("Step took {} ms",start.elapsed().as_millis());
    });
    wind.add(&btn_step);
    let btn_stepref = Rc::new(RefCell::new(btn_step));
 
    let intervall = Rc::new(RefCell::new(INITIALUPDATEINTERVALL));
    let mut inp_update_intervall = FloatInput::default().with_size(100, 20).below_of(&mnu_shapeselect, 5).with_label("Update Intervall:");
    let mnu_shapeselectref = Rc::new(RefCell::new(mnu_shapeselect));

    inp_update_intervall.set_value(format!("{}",INITIALUPDATEINTERVALL).as_str());
    wind.add(&inp_update_intervall);
    let inp_update_intervallref = Rc::new(RefCell::new(inp_update_intervall));
    let btn_drawchunksref = Rc::new(RefCell::new(btn_drawchunks));

    let mut timeouthandle = app::add_timeout3(core::f64::MAX, |_|()); //<- i despise this. creates dummy timer just to fill timeouthandle

    //TODO make better use of drawmode (maybe remove from field?)
    let canvasref0 = canvasref.clone();
    let inp_update_intervallref1 = inp_update_intervallref.clone();
    let intervallref1 = intervall.clone();
    let btn_stepref1 = btn_stepref.clone(); 
    let mnu_shapeselectref1 = mnu_shapeselectref.clone();

    btn_stop_toggle.set_callback(move |handle| {
        if handle.value() {
            remove_timeout3(timeouthandle);
            //println!("timer destroyed");

            *canvasref0.borrow_mut().drawmoderef.borrow_mut() = true;
            inp_update_intervallref1.borrow_mut().set_value(format!{"{}",intervallref1.borrow()}.as_str());
            inp_update_intervallref1.borrow_mut().show();
            btn_stepref1.borrow_mut().show();
            mnu_shapeselectref.borrow_mut().show();
            handle.set_label("Start");
        }
        else {
            let oldintervall = *intervallref1.borrow();
            let newintervall = inp_update_intervallref1.borrow().value().parse().unwrap_or(oldintervall);
            if 0.0 <= newintervall && newintervall <= 10.0 {
                intervallref1.replace(newintervall);
            }
            *canvasref0.borrow_mut().drawmoderef.borrow_mut() = false;
            inp_update_intervallref1.borrow_mut().hide();
            btn_stepref1.borrow_mut().hide();
            mnu_shapeselectref.borrow_mut().hide();
            handle.set_label("Stop");
            //------------------------------------
            //let btn_stop_toggleref1 = btn_stop_toggleref.clone();
            //let inp_update_intervallref2 = inp_update_intervallref.clone();
            let canvasref1 = canvasref0.clone();
            let intervallref2 = intervallref1.clone();

            let update = move |handle| {        
                let start = std::time::Instant::now();
                let fieldref = &canvasref1.borrow_mut().field;
                fieldref.borrow_mut().update_threaded();

                let secs = start.elapsed().as_secs_f64();
                println!("elapsed time is {} s, setting timout {} s\n{} alive chunks", secs,*intervallref2.borrow()-secs,fieldref.borrow().vec.len());

                app::repeat_timeout3(*intervallref2.borrow()-start.elapsed().as_secs_f64(), handle);
            };
        
            timeouthandle = app::add_timeout3(*intervall.borrow(), update);
            //println!("timer started for the first time with timeout {}", *intervall.borrow() );
        
            //------------------------------------
        }
    }); //TODO perhaps add btn to canvas-type
    wind.add(&btn_stop_toggle);
    let btn_stop_toggleref = Rc::new(RefCell::new(btn_stop_toggle));

    let mut lbl_coords = TextDisplay::new(0,HEIGHT,100,0,"");
    lbl_coords.set_frame(FrameType::NoBox); //<- i hate this
    wind.add(&lbl_coords);

    //wind.end();
    wind.show();
    
    let canvasref2 = canvasref.clone();
    let btn_stop_toggleref2 = btn_stop_toggleref.clone();
    let btn_stepref2 = btn_stepref.clone();

    let mut starttime_tick = std::time::Instant::now();
    let tick = move |handle| {
        let xoffset = *canvasref2.borrow().xoffsetref.borrow();
        let yoffset = *canvasref2.borrow().yoffsetref.borrow();
        let linedist = *canvasref2.borrow().linedistref.borrow();

        //TODO find better way to redraw stuff, use damage values to determine what needs to be redrawn
        canvasref2.borrow_mut().redraw_canvas(btn_drawchunksref.borrow().value()); //TODO this can take long, i could collect the time it takes or smth and substract it from the update intervall
        btn_stop_toggleref2.borrow_mut().redraw();
        btn_stepref2.borrow_mut().redraw();
        //println!("updated canvas");
        //println!("redrew canvas in {} ms", start.elapsed().as_millis());

        let xmod = (app::event_x()+xoffset).rem_euclid(linedist);
        let ymod = (app::event_y()+yoffset).rem_euclid(linedist);

        let curcellmousepos = (((app::event_x()+xoffset-xmod)/linedist),((app::event_y()+yoffset-ymod)/linedist));

        lbl_coords.set_label(format!("X: {} Y: {}", curcellmousepos.0, curcellmousepos.1).as_str());

        app::repeat_timeout3(TICKTIME-starttime_tick.elapsed().as_secs_f64(), handle);
        starttime_tick = std::time::Instant::now();
    };

    app::add_timeout3(TICKTIME, tick);
    
    app.run().unwrap();
}