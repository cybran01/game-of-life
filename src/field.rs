use std::collections::HashMap;

#[derive(Clone)]
pub struct Square {
    size: usize,
    cell: Vec<Vec<bool>>,
    alive_cells: i32,
}
#[derive(Clone)]
pub struct Field {
    pub vec: HashMap<(isize, isize), Square>,
    chunksize: usize,
}
enum Direction {
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl Direction {
    fn shift(coords: (isize, isize), dir: &Direction) -> (isize, isize) {
        match dir {
            Direction::North => (coords.0, coords.1 + 1),
            Direction::South => (coords.0, coords.1 - 1),
            Direction::West => (coords.0 - 1, coords.1),
            Direction::East => (coords.0 + 1, coords.1),
            Direction::NorthWest => (coords.0 - 1, coords.1 + 1),
            Direction::NorthEast => (coords.0 + 1, coords.1 + 1),
            Direction::SouthWest => (coords.0 - 1, coords.1 - 1),
            Direction::SouthEast => (coords.0 + 1, coords.1 - 1),
        }
    }

    fn iter() -> std::slice::Iter<'static, Direction> {
        use self::Direction::*;
        static DIRECTIONS: [Direction; 8] = [
            North, South, East, West, NorthWest, NorthEast, SouthWest, SouthEast,
        ];
        DIRECTIONS.iter()
    }
}

impl Square {
    fn new(size: usize) -> Self {
        //initializes a Square with all cells dead
        let cells = vec![vec![false; size]; size];
        Self {
            size,
            cell: cells,
            alive_cells: 0,
        }
    }
    fn get_cell(&self, x: usize, y: usize) -> bool {
        assert!((0..self.size).contains(&x));
        assert!((0..self.size).contains(&y));

        self.cell[x][y]
    }
    fn set_cell(&mut self, x: usize, y: usize, v: bool) {
        assert!((0..self.size).contains(&x));
        assert!((0..self.size).contains(&y));

        if self.cell[x][y] && !v {
            self.alive_cells -= 1;
        } else if !self.cell[x][y] && v {
            self.alive_cells += 1;
        }
        self.cell[x][y] = v;
    }
}

pub type Shape = Vec<Vec<Option<bool>>>;

impl Field {
    pub fn new(chunksize: usize) -> Self {
        let vec = HashMap::new();
        Self { vec, chunksize }
    }

    pub fn get_cell(&self, x: isize, y: isize) -> bool {
        let coord_in_square = (
            x.rem_euclid(self.chunksize as isize) as usize,
            y.rem_euclid(self.chunksize as isize) as usize,
        );
        let square_coord = (
            (x - coord_in_square.0 as isize) / self.chunksize as isize,
            (y - isize::try_from(coord_in_square.1).unwrap()) / self.chunksize as isize,
        );

        match self.vec.get(&square_coord) {
            Some(square) => square.get_cell(coord_in_square.0, coord_in_square.1),
            None => false,
        }
    }

    pub fn set_cell(&mut self, coords: (isize, isize), val: bool) {
        let localcoords = (
            coords.0.rem_euclid(self.chunksize as isize),
            coords.1.rem_euclid(self.chunksize as isize),
        );
        let squarecoords = (
            (coords.0 - localcoords.0) / self.chunksize as isize,
            (coords.1 - localcoords.1) / self.chunksize as isize,
        );

        match self.vec.get_mut(&squarecoords) {
            Some(cursquare) => {
                cursquare.set_cell(localcoords.0 as usize, localcoords.1 as usize, val);
                if cursquare.alive_cells == 0 {
                    self.vec.remove(&squarecoords);
                }
            }
            None => {
                if val {
                    let mut cursquare = Square::new(self.chunksize);
                    cursquare.set_cell(localcoords.0 as usize, localcoords.1 as usize, true);
                    self.vec.insert(squarecoords, cursquare);
                }
            }
        }
    }

    pub fn set_shape_at(&mut self, coords: (isize, isize), shape: &Shape) {
        //takes vector of columns
        for (x, line) in shape.iter().enumerate() {
            for (y, item) in line.iter().enumerate() {
                if let Some(val) = item {
                    self.set_cell((coords.0 + x as isize, coords.1 + y as isize), *val);
                }
            }
        }
    }

    fn eval_cells_alive_on_boundary(&self, x: isize, y: isize) -> i32 {
        //TODO maybe do this via maxdist of x,y values
        let mut counter = 0;

        for dir in Direction::iter() {
            let coords = Direction::shift((x, y), dir);
            if self.get_cell(coords.0, coords.1) {
                counter += 1;
            }
        }
        counter
    }

    pub fn update_chunk(&self, coords: (isize, isize)) -> Option<Square> {
        let cursquare = self.vec.get(&coords);
        let mut square;
        let mut checkonlyboundary = false;

        match cursquare {
            Some(squareref) => square = squareref.clone(),
            None => {
                square = Square::new(self.chunksize);
                checkonlyboundary = true;
            }
        }
        for x in 0..self.chunksize {
            for y in 0..self.chunksize {
                if !checkonlyboundary
                    || x == 0
                    || y == 0
                    || x == self.chunksize - 1
                    || y == self.chunksize - 1
                {
                    let curcoord = (
                        coords.0 * self.chunksize as isize + x as isize,
                        coords.1 * self.chunksize as isize + y as isize,
                    );

                    let cells_alive_on_boundary =
                        self.eval_cells_alive_on_boundary(curcoord.0, curcoord.1);
                    let curcell = square.get_cell(x, y);

                    if !curcell && cells_alive_on_boundary == 3 {
                        //a dead cell with 3 neighbors becomes alive
                        square.set_cell(x, y, true);
                    } else if curcell
                        && !(cells_alive_on_boundary == 2 || cells_alive_on_boundary == 3)
                    {
                        //an alive cell that does not have 2 or 3 alive neighbors, dies
                        square.set_cell(x, y, false);
                    }
                }
            }
        }
        if square.alive_cells == 0 {
            None
        } else {
            Some(square)
        }
    }

    fn insert_valid_only(
        key: (isize, isize),
        elem: Option<Square>,
        hs: &mut HashMap<(isize, isize), Square>,
    ) {
        //TODO check if try_insert is finally out of nightly
        if let Some(value) = elem {
            hs.entry(key).or_insert(value);
        }
    }

    pub fn update(&mut self) {
        let allkeys = self.vec.clone().into_keys();
        self.vec = self.update_keys(allkeys);
    }

    pub fn update_keys<T: IntoIterator<Item = (isize, isize)>>(
        &self,
        keys: T,
    ) -> HashMap<(isize, isize), Square> {
        let mut hs = HashMap::new();

        for (x, y) in keys {
            let newchunk = self.update_chunk((x, y));
            Self::insert_valid_only((x, y), newchunk, &mut hs);

            for dir in Direction::iter() {
                let coords = Direction::shift((x, y), dir);
                let newchunk = self.update_chunk(coords);
                Self::insert_valid_only(coords, newchunk, &mut hs);
            }
            //The following is not possible anymore when using parallel code
            //self.vec.remove_entry(&(x,y));
        }
        hs
    }
    pub fn update_threaded(&mut self, chunks_per_thread: usize, max_threads: usize) {
        let allkeys = self.vec.clone().into_keys();

        use std::cmp::{max, min};
        let threadnum = min(
            max(allkeys.size_hint().0 / chunks_per_thread, 1),
            max_threads,
        );
        println!(
            "At least {} chunks, hence {} thread(s) to spawn",
            allkeys.size_hint().0,
            min(
                max(allkeys.size_hint().0 / chunks_per_thread, 1),
                max_threads
            )
        );

        let mut splitkeysvec = vec![Vec::new(); threadnum];
        for (i, x) in allkeys.enumerate() {
            splitkeysvec.get_mut(i % threadnum).unwrap().push(x);
        }

        let mut res = Vec::new();

        use std::thread::scope;
        scope(|s| {
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

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}
