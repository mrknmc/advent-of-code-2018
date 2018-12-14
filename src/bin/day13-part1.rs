use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::VecDeque;

#[derive(Debug)]
enum Turn {
    Left,
    Right,
    Straight,
}

impl Turn {
    fn angle(&self) -> i32 {
        match self {
            Turn::Left => 90,
            Turn::Right => -90,
            Turn::Straight => 0
        }
    }
}

#[derive(Debug)]
struct World {
    cars: Vec<Car>,
    tracks: Vec<Vec<Track>>,
}

impl World { 

    fn has_crash(&self) -> bool {
        self.cars.iter().enumerate()
            .any(|(c1_id, c1)| self.cars.iter().enumerate()
                .any(|(c2_id, c2)| c1_id != c2_id && c1.crashed_with(&c2)))
    }

    fn crash_location(&self) -> Option<(usize, &Car)> {
        self.cars.iter().enumerate()
            .find(|(c1_id, c1)| self.cars.iter().enumerate()
                .any(|(c2_id, c2)| c1_id != &c2_id && c1.crashed_with(&c2)))
    }

    fn tick(&mut self) {
        for mut car in self.cars.as_mut_slice() {
            let track = &self.tracks[car.x][car.y];
            car.tick(&track);
        }
    }
}

#[derive(Debug, Clone)]
enum Track {
    Empty,
    Horizontal,
    Vertical,
    Intersection,
    CurveForward,
    CurveBackward
}

#[derive(Debug, Clone)]
enum Direction {
    FaceUp,
    FaceDown,
    FaceLeft,
    FaceRight,
}

impl Direction {

    fn angle(&self) -> i32 {
        match self {
            Direction::FaceUp => 90,
            Direction::FaceDown => -90,
            Direction::FaceLeft => 180,
            Direction::FaceRight => 0,
        }
    }

    fn from_angle(angle: i32) -> Direction {
        match angle {
            -360 => Direction::FaceRight,
            -270 => Direction::FaceUp,
            -180 => Direction::FaceRight,
            -90 => Direction::FaceDown,
            0 => Direction::FaceRight,
            90 => Direction::FaceUp,
            180 => Direction::FaceLeft,
            270 => Direction::FaceDown,
            360 => Direction::FaceRight,
            _ => { panic!("Unexpected number of degrees: {}", angle); }
        }
    }
    
    fn apply(&self, turn: &Turn) -> Direction {
        Self::from_angle(self.angle() + turn.angle() % 360)
    }
}

#[derive(Debug)]
struct Car {
    x: usize,
    y: usize,
    direction: Direction,
    turns: VecDeque<Turn>,
}

impl Car {

    fn new(x: usize, y: usize, direction: Direction) -> Car {
        Car { x, y, direction, turns: vec!(Turn::Left, Turn::Straight, Turn::Right).into_iter().collect() }
    }

    fn move_in_direction(&mut self, direction: &Direction) {
        let (x, y) = match direction {
            Direction::FaceUp => (0, -1),
            Direction::FaceDown => (0, 1),
            Direction::FaceLeft => (-1, 0),
            Direction::FaceRight => (1, 0),
        };
        self.x += x;
        self.y += y;
    }

    fn crashed_with(&self, other: &Car) -> bool {
        (self.x, self.y) == (other.x, other.y)
    }

    fn tick(&mut self, track: &Track) {
        let new_direction = match (self.direction, track) {
            (_, Track::Empty) => { panic!("Car not on track!"); }
            (Direction::FaceDown, Track::Horizontal) | (Direction::FaceUp, Track::Horizontal) => { panic!("Car verical on horizontal track"); },
            (Direction::FaceLeft, Track::Vertical) | (Direction::FaceRight, Track::Vertical) => { panic!("Car horizontal on vertical track"); },
            (Direction::FaceUp, Track::CurveForward) => {
                // self.move_by(1, 0);
                Direction::FaceRight
            }
            (Direction::FaceUp, Track::CurveBackward) => {
                // self.move_by(-1, 0);
                Direction::FaceLeft
            },
            (Direction::FaceDown, Track::CurveForward) => {
                // self.move_by(-1, 0);
                Direction::FaceLeft
            }
            (Direction::FaceDown, Track::CurveBackward) => {
                // self.move_by(1, 0);
                Direction::FaceRight
            }
            (Direction::FaceRight, Track::CurveForward) => {
                // self.move_by(0, -1);
                Direction::FaceUp
            }
            (Direction::FaceRight, Track::CurveBackward) => {
                // self.move_by(0, 1);
                Direction::FaceDown
            }
            (Direction::FaceLeft, Track::CurveForward) => {
                // self.move_by(0, 1);
                Direction::FaceDown
            }
            (Direction::FaceLeft, Track::CurveBackward) => {
                // self.move_by(0, -1);
                Direction::FaceUp
            }
            (direction, Track::Vertical) | (direction, Track::Horizontal) => {
                // self.move_by(1, 0);
                direction
            }
            (direction, Track::Intersection) => {
                let turn = self.turns.pop_back().unwrap();
                let new_direction = direction.apply(&turn);
                self.turns.push_front(turn);
                new_direction
            }
        };
        self.move_in_direction(&new_direction);
        self.direction = new_direction;
    }
}

fn read_file<'a>(filename: String) -> World {
    let f = File::open(filename).expect("input file not found");
    let buf_reader = BufReader::new(f);
    let lines = buf_reader.lines().map(|s| s.unwrap());
    let mut cars = Vec::new();
    let mut tracks: Vec<Vec<Track>> = vec![vec![Track::Empty; 150]; 150];
    for (row, line) in lines.enumerate() {
        for (col, c) in line.chars().enumerate() {
            match c {
                ' ' => { continue; },
                '-' => { tracks[col][row] = Track::Horizontal; },
                '|' => { tracks[col][row] = Track::Vertical; }
                '/' => { tracks[col][row] = Track::CurveForward; },
                '\\' => { tracks[col][row] = Track::CurveBackward; },
                '+' => { tracks[col][row] = Track::Intersection; },
                'v' => { cars.push(Car::new(col, row, Direction::FaceDown)); },
                '^' => { cars.push(Car::new(col, row, Direction::FaceUp)); },
                '>' => { cars.push(Car::new(col, row, Direction::FaceRight)); },
                '<' => { cars.push(Car::new(col, row, Direction::FaceLeft)); },
                c => { panic!("Unknown character: {}", c); }
            }
        }
    }

    // Add tracks to where the cars are
    for car in &cars {
        tracks[car.x][car.y] = match (
            &tracks[car.x - 1][car.y],
            &tracks[car.x + 1][car.y],
            &tracks[car.x][car.y - 1],
            &tracks[car.x][car.y + 1]
        ) {
            // left, right, top, bottom
            (Track::Horizontal, Track::Horizontal, Track::Vertical, Track::Vertical) => Track::Intersection,
            (Track::Horizontal, Track::Horizontal, _, _) => Track::Horizontal,
            (_, _, Track::Vertical, Track::Vertical) => Track::Vertical,
            (Track::Horizontal, _, Track::Vertical, _) => Track::CurveForward,
            (Track::Horizontal, _, _, Track::Vertical) => Track::CurveBackward,
            (_, Track::Horizontal, Track::Vertical, _) => Track::CurveBackward,
            (_, Track::Horizontal, _, Track::Vertical) => Track::CurveForward,
            (Track::Intersection, Track::Intersection, Track::Intersection, Track::Intersection) => Track::Vertical,
            (Track::Intersection, Track::Intersection, _, _) => Track::Horizontal,
            (Track::Intersection, Track::Horizontal, _, _) => Track::Horizontal,
            (Track::Horizontal, Track::Intersection, _, _) => Track::Horizontal,
            (_, _, Track::Intersection, Track::Intersection) => Track::Vertical,
            (_, _, Track::Intersection, Track::Vertical) => Track::Vertical,
            (_, _, Track::Vertical, Track::Intersection) => Track::Vertical,
            pattern => { panic!("Unrecognized pattern around car: {:?} on coords: {}, {}", pattern, car.x, car.y); }
        };
    }

    World { cars, tracks, }
}


fn main() {
    let filename = env::args().nth(1).expect("No argument filename passed");
    let mut world = read_file(filename);
    while !world.has_crash() {
        world.tick();
    }
    println!("{:?}", world.crash_location());
}