const DIM:usize = 32;

use std::{cell::Cell, borrow::Borrow};
use std::rc::Rc;

// pub trait Neighbor {
//     fn neighbor_east() -> Self;
//     fn neighbor_west() -> Self;
//     fn neighbor_north() -> Self;
//     fn neighbor_east() -> Self;
//     fn neighbor_east() -> Self;
//     fn neighbor_east() -> Self;
//     fn neighbor_east() -> Self;
//     fn neighbor_east() -> Self;
// }

//TODO allow certain square shapes that are simpler and have less cells such that they can be stored more efficiently after the update method is called

pub trait CellsGettable {
    fn get_cell(&self,x:usize,y:usize) -> bool;
}
#[derive(Clone)]
struct ThinSquareLine {
    cell: [bool;DIM]
}
#[derive(Clone)]
struct ThinSquareCorner {
    leftline:[bool;DIM],
    rightline:[bool;DIM-1]
}
#[derive(Clone)]
struct FullSquare {
    cell: [[bool;DIM];DIM],
    alive_cells: i32
}
#[derive(Clone)]
enum Square {
    Full(FullSquare),
    NorthBoundary(ThinSquareLine),
    SouthBoundary(ThinSquareLine),
    WestBoundary(ThinSquareLine),
    EastBoundary(ThinSquareLine),
    NorthWestBoundary(ThinSquareCorner),
    NorthEastBoundary(ThinSquareCorner),
    SouthWestBoundary(ThinSquareCorner),
    SouthEastBoundary(ThinSquareCorner)
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

impl FullSquare {
    fn getBoundary(&self, dir:&Direction) -> Square {
        match dir {
            Direction::North => FullSquare::getNorthBoundary(self),
            Direction::South => FullSquare::getSouthBoundary(self),
            Direction::West => FullSquare::getWestBoundary(self),
            Direction::East => FullSquare::getEastBoundary(self),
            Direction::NorthWest => FullSquare::getNorthWestBoundary(self),
            Direction::NorthEast => FullSquare::getNorthEastBoundary(self),
            Direction::SouthWest => FullSquare::getSouthWestBoundary(self),
            Direction::SouthEast => FullSquare::getSouthEastBoundary(self)
        }
    }
    fn getNorthBoundary(&self) -> Square {
        let mut cell:[bool;DIM] = Default::default();
        for x in 0..DIM {
            cell[x]=self.cell[x][DIM-1];
        }
        Square::NorthBoundary(ThinSquareLine{cell:cell})
    }
    fn getSouthBoundary(&self) -> Square {
        let mut cell:[bool;DIM] = Default::default();
        for x in 0..DIM {
            cell[x]=self.cell[x][0];
        }
        Square::SouthBoundary(ThinSquareLine{cell:cell})
    }
    fn getWestBoundary(&self) -> Square {
        Square::WestBoundary(ThinSquareLine{cell:self.cell[0]})
    }
    fn getEastBoundary(&self) -> Square {
        Square::EastBoundary(ThinSquareLine{cell:self.cell[DIM-1]})
    }
    fn getNorthWestBoundary(&self) -> Square {
        let westcells:[bool;DIM] = self.cell[0];
        let mut northcells:[bool;DIM-1] = Default::default();
        for x in 1..DIM {
            northcells[x-1]=self.cell[x][DIM-1];
        }
        Square::NorthWestBoundary(ThinSquareCorner{leftline:westcells,rightline:northcells})
    }
    fn getNorthEastBoundary(&self) -> Square {
        let mut northcells:[bool;DIM] = Default::default();
        for x in 0..DIM {
            northcells[x]=self.cell[x][DIM-1];
        }
        let eastcells:[bool;DIM-1] = self.cell[DIM-1][0..DIM-1].try_into().unwrap();
        Square::NorthEastBoundary(ThinSquareCorner{leftline:northcells,rightline:eastcells})
        
    }
    fn getSouthWestBoundary(&self) -> Square {
        let mut southcells:[bool;DIM] = Default::default();
        for x in 0..DIM {
            southcells[x]=self.cell[x][0];
        }
        let westcells:[bool;DIM-1] = self.cell[0][1..DIM].try_into().unwrap();
        Square::SouthWestBoundary(ThinSquareCorner{leftline:southcells,rightline:westcells})
    }
    fn getSouthEastBoundary(&self) -> Square {
        let eastcells:[bool;DIM] = self.cell[DIM-1];
        let mut southcells:[bool;DIM-1] = Default::default();
        for x in 0..DIM-1 {
            southcells[x]=self.cell[x][0];
        }
        Square::SouthEastBoundary(ThinSquareCorner{leftline:eastcells,rightline:southcells})
    }
}

impl CellsGettable for Square {
    fn get_cell(&self,x:usize,y:usize) -> bool {
        assert!((0..DIM).contains(&x));
        assert!((0..DIM).contains(&y));

        match self {
            Square::Full(square) => {
                square.cell[x][y]
            },
            Square::NorthBoundary(square) => {
                if y == DIM-1 {
                    square.cell[x]
                }
                else {
                    false
                }
            },
            Square::SouthBoundary(square) => {
                if y == 0 {
                    square.cell[x]
                }
                else {
                    false
                }
            },
            Square::WestBoundary(square) => {
                if x == 0 {
                    square.cell[y]
                }
                else {
                    false
                }
            },
            Square::EastBoundary(square) => {
                if x == DIM-1 {
                    square.cell[y]
                }
                else {
                    false
                }
            },
            Square::NorthWestBoundary(square) => {
                if x == 0 {
                    square.leftline[y]
                }
                else if y == DIM-1 {
                    square.rightline[x-1]
                }
                else {
                    false
                }
            },
            Square::NorthEastBoundary(square) => {
                if y == DIM-1 {
                    square.leftline[x]
                }
                else if x == DIM-1 {
                    square.rightline[y]
                }
                else {
                    false
                }
            },
            Square::SouthWestBoundary(square) => {
                if y == 0 {
                    square.leftline[x]
                }
                else if x == 0 {
                    square.rightline[y-1]
                }
                else {
                    false
                }
            },
            Square::SouthEastBoundary(square) => {
                if x == DIM-1 {
                    square.leftline[y]
                }
                else if y == 0 {
                    square.rightline[x]
                }
                else {
                    false
                }
            }
        }
    }
}

impl FullSquare {
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
    fn get_cell(&self,x:isize,y:isize) -> bool {
        let dim = isize::try_from(DIM).unwrap();

        let coord_in_square = (usize::try_from(x.rem_euclid(DIM.try_into().unwrap())).unwrap(),usize::try_from(y.rem_euclid(DIM.try_into().unwrap())).unwrap());
        let square_coord = ((x-isize::try_from(coord_in_square.0).unwrap())/dim, (y-isize::try_from(coord_in_square.1).unwrap())/dim);
        
        match self.vec.get(&square_coord) {
            Some(square) => square.get_cell(coord_in_square.0,coord_in_square.1),
            None => false
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
            Some(Square::Full(squareref)) => square = squareref.clone(),
            None => {
                square = FullSquare{cell:[[false;DIM];DIM],alive_cells:0};
                checkonlyboundary = true;
            },
            Some (_) => {
                return None;
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
            return Some(Square::Full(square));
        }
    }

    fn insert_valid_only(key:(isize,isize), elem:Option<Square>, hs:&mut std::collections::HashMap<(isize,isize),Square>) {
        if let Some(Square::Full(_)) = elem {
                if !hs.contains_key(&key) {
                    hs.insert(key, elem.unwrap());
                }
        }
    }

    fn update (&mut self) {
        let mut hs:std::collections::HashMap<(isize,isize),Square> = std::collections::HashMap::new();

        let keys;
        let binding = self.vec.clone();
        keys = binding.keys(); 

        for (x,y) in keys {

            let newchunk = self.update_chunk((*x,*y));
            Self::insert_valid_only((*x,*y),newchunk,&mut hs);

            for dir in Direction::iter() {
                let coords = Direction::shift((*x,*y), dir);
                let newchunk = self.update_chunk(coords);
                Self::insert_valid_only(coords,newchunk,&mut hs);
            }

            self.vec.remove_entry(&(*x,*y));

            for dir in Direction::iter() {
                let coords = Direction::shift((*x,*y), dir);
                let cursquare = self.vec.get(&coords);
                if let Some(Square::Full(square)) = cursquare {
                    self.vec.insert(coords,square.getBoundary(dir));
                }
            }
        }
        self.vec = hs;
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

/* fn main() {
     let glider = [[false,true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [true,true,true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false]];
    let start = Square::Full(FullSquare {cell:glider,alive_cells:5});
    let mut hashmap = std::collections::HashMap::new();
    hashmap.insert((0,0), start);
    let mut f = Field{vec:hashmap};

    for _i in  0..121 {
        print(&f);
        println!();
        f.update(); 
    } 

}
 */

 use fltk::{
    app,
    draw::{draw_line, draw_point, draw_rect_fill, set_draw_color, set_line_style, LineStyle},
    enums::{Color, Event, FrameType},
    frame::Frame,
    prelude::*,
    surface::ImageSurface,
    window::Window,
};
use std::cell::RefCell;
struct Canvas {
    frame: Frame,
    #[allow(dead_code)]
    surf: Rc<RefCell<ImageSurface>>,
    field:Rc<RefCell<Field>>,
    xoffsetref:Rc<RefCell<i32>>,
    yoffsetref:Rc<RefCell<i32>>
}

impl Canvas {
    pub fn new(w: i32, h: i32, field: Rc<RefCell<Field>>,xoffset:i32,yoffset:i32) -> Self {
        let mut frame = Frame::default().with_size(w, h).center_of_parent();
        frame.set_color(Color::White);
        frame.set_frame(FrameType::DownBox);

        let xoffsetref = Rc::new(RefCell::new(xoffset));
        let yoffsetref = Rc::new(RefCell::new(yoffset));

        let surf = ImageSurface::new(frame.width(), frame.height(), false);
        ImageSurface::push_current(&surf);
        draw_rect_fill(0, 0, w, h, Color::White);

        set_draw_color(Color::Black);
        for xcoord in 0..WIDTH/LINEDIST { //ugly
            draw_line(xcoord*LINEDIST, 0, xcoord*LINEDIST, HEIGHT);
        }
        for ycoord in 0..HEIGHT/LINEDIST { //ugly
            draw_line(0, ycoord*LINEDIST, WIDTH, ycoord*LINEDIST);
        }
        
        for xcoord in 0..WIDTH/LINEDIST { //ugly
            for ycoord in 0..HEIGHT/LINEDIST { //ugly
                if field.borrow_mut().get_cell(xcoord as isize, ycoord as isize) {
                    //roles of x and y are swapped due to GUI convention
                    draw_rect_fill(ycoord*LINEDIST, xcoord*LINEDIST,LINEDIST, LINEDIST,Color::Black);
                }
            }
        }
        ImageSurface::pop_current();

        let surf = Rc::from(RefCell::from(surf));

        frame.draw({
            let surf = surf.clone();
            move |f| {
                let surf = surf.borrow_mut();
                let mut img = surf.image().unwrap();
                img.draw(f.x(), f.y(), f.w(), f.h());
            }
        });

        frame.handle({
            let mut x = 0;
            let mut y = 0;
            let surf = surf.clone();
            let field = field.clone();
            let xoffsetref = xoffsetref.clone();
            let yoffsetref = yoffsetref.clone();

            move |f, ev| {
                let mut surf = surf.borrow_mut(); //why not just borrow()?
                let mut field = field.borrow_mut(); //why not just borrow()?

                match ev {
                    Event::Push => {
                        println!("Push");
                        let coords = app::event_coords();
                        x = coords.0;
                        y = coords.1;
                        true
                    }
                    Event::Drag => {
                        println!("Drag");
                        let coords = app::event_coords();

                        //mirrored bc of gui convention
                        let newyoffset = (*yoffsetref.borrow_mut()-(coords.0-x));
                        let newxoffset = (*xoffsetref.borrow_mut()-(coords.1-y));

                        yoffsetref.replace(newyoffset);
                        xoffsetref.replace(newxoffset);
                        //f.redraw();

                        //Canvas::update_field_no_ref(f,&mut surf,&mut field,newxoffset,newyoffset); //TODO cleanly update after drag

                        x = coords.0;
                        y = coords.1;
                        true
                    }
                    _ => false,
                }
            }
        });
        Self { frame, surf , field, xoffsetref, yoffsetref}
    }

    fn redraw_canvas_no_ref(frame:&mut Frame, surf:&mut ImageSurface, field:&mut Field, xoffset:i32, yoffset:i32) {
        let xmod = LINEDIST-xoffset.rem_euclid(LINEDIST);
        let ymod = LINEDIST-yoffset.rem_euclid(LINEDIST);
        
        ImageSurface::push_current(&surf);
        draw_rect_fill(0, 0, WIDTH, HEIGHT, Color::White);
        
        set_draw_color(Color::Black);

        for xcoord in 0..WIDTH/LINEDIST+1 { //ugly, verify that the +1 ensures enough lines are always visible despite mod
            draw_line(xcoord*LINEDIST+ymod, 0, xcoord*LINEDIST+ymod, HEIGHT);
        }
        for ycoord in 0..HEIGHT/LINEDIST+1 { //ugly, verify that the +1 ensures enough lines are always visible despite mod
            draw_line(0, ycoord*LINEDIST+xmod, WIDTH, ycoord*LINEDIST+xmod);
        }
        
        for xcoord in -1..HEIGHT/LINEDIST+1 { //ugly, verify that the -1/+1 ensures enough lines are always visible despite mod
            for ycoord in -1..WIDTH/LINEDIST+1 { //ugly, verify that the 1/+1 ensures enough lines are always visible despite mod
                if field.get_cell((xcoord+xoffset/LINEDIST) as isize, (ycoord+yoffset/LINEDIST) as isize) {
                    //roles of x and y are swapped due to GUI convention
                    draw_rect_fill(ycoord*LINEDIST+ymod, xcoord*LINEDIST+xmod,LINEDIST, LINEDIST,Color::Black);
                }
            }
        }
        ImageSurface::pop_current();
        frame.redraw();
    }

    fn redraw_canvas(&mut self) {
        let x:i32 = *self.xoffsetref.borrow_mut();
        let y:i32 = *self.yoffsetref.borrow_mut();
        let frame:&mut Frame = &mut self.frame;
        let surf:&mut ImageSurface = &mut *self.surf.borrow_mut();
        let field:&mut Field = &mut *self.field.borrow_mut();

        Canvas::redraw_canvas_no_ref(frame, surf, field, x, y);
    } 
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const LINEDIST: i32 = 30;
const TICKTIME: f64 = 0.1; 
const UPDATEINTERVALL: f64 = 0.1; 
const XSTARTOFFSET: i32 = 0;
const YSTARTOFFSET: i32 = 0;

fltk::widget_extends!(Canvas, Frame, frame);

fn main() {
    //-----------------------------------------------------------------------------------------
    let glider = [[false,false,true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [true,false,true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,true,true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],
                                    [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false]];
    
    let start = Square::Full(FullSquare {cell:glider,alive_cells:5});
    let mut hashmap = std::collections::HashMap::new();
    hashmap.insert((0,0), start);
    let field = Field{vec:hashmap};
    //-----------------------------------------------------------------------------------------

    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Game of Life");
    
    let f = RefCell::new(field);
    let canvas = Canvas::new(WIDTH, HEIGHT, f.into(), XSTARTOFFSET, YSTARTOFFSET);

    wind.end();
    wind.show();


    let fieldref = canvas.field.clone();
    let update = move |handle| {
        fieldref.borrow_mut().update();
        println!("updateded field");
        app::repeat_timeout3(UPDATEINTERVALL, handle);
    };

    app::add_timeout3(UPDATEINTERVALL, update);

    let canvasref = Rc::new(RefCell::new(canvas));
    let tick = move |handle| {
        canvasref.borrow_mut().redraw_canvas();
        println!("updated canvas");
        app::repeat_timeout3(TICKTIME, handle);
    };

    app::add_timeout3(TICKTIME, tick);

    app.run().unwrap();
}