use bevy::prelude::*;
use serde::{Serialize, Deserialize};

pub const CELL: f32 = 2.0;
pub const DEFAULT_GRID_W: u32 = 12;
pub const DEFAULT_GRID_H: u32 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CellKind {
    #[default]
    Empty,
    Wall,
    Player,
    Box,
    Target,
    Key,
    Gate,
    Decoration,
}

#[derive(Resource, Clone)]
pub struct GridData {
    pub cells: Vec<Vec<CellKind>>,
    pub width: u32,
    pub height: u32,
    pub version: u64,
}

impl Default for GridData {
    fn default() -> Self {
        let w = DEFAULT_GRID_W as usize;
        let h = DEFAULT_GRID_H as usize;
        let mut cells = vec![vec![CellKind::Empty; h]; w];
        for i in 0..w {
            for j in 0..h {
                if i == 0 || j == 0 || i == w - 1 || j == h - 1 {
                    cells[i][j] = CellKind::Wall;
                }
            }
        }
        cells[5][5] = CellKind::Player;
        cells[6][5] = CellKind::Box;
        cells[6][6] = CellKind::Target;
        cells[8][8] = CellKind::Key;
        cells[8][7] = CellKind::Gate;
        Self { cells, width: DEFAULT_GRID_W, height: DEFAULT_GRID_H, version: 0 }
    }
}

impl GridData {
    pub fn get(&self, x: i32, z: i32) -> CellKind {
        if x < 0 || z < 0 || x >= self.width as i32 || z >= self.height as i32 {
            CellKind::Empty
        } else {
            self.cells[x as usize][z as usize]
        }
    }

    pub fn set(&mut self, x: i32, z: i32, kind: CellKind) {
        if x >= 0 && z >= 0 && x < self.width as i32 && z < self.height as i32 {
            self.cells[x as usize][z as usize] = kind;
            self.version += 1;
        }
    }

    pub fn resize(&mut self, new_w: u32, new_h: u32) {
        let nw = new_w as usize;
        let nh = new_h as usize;
        let mut new_cells = vec![vec![CellKind::Empty; nh]; nw];
        for x in 0..nw.min(self.cells.len()) {
            let row = &self.cells[x];
            for z in 0..nh.min(row.len()) {
                new_cells[x][z] = row[z];
            }
        }
        for i in 0..nw {
            for j in 0..nh {
                if i == 0 || j == 0 || i == nw - 1 || j == nh - 1 {
                    if new_cells[i][j] == CellKind::Empty {
                        new_cells[i][j] = CellKind::Wall;
                    }
                }
            }
        }
        self.cells = new_cells;
        self.width = new_w;
        self.height = new_h;
        self.version += 1;
    }
}

pub fn find_player(cells: &[Vec<CellKind>]) -> (i32, i32) {
    for (x, row) in cells.iter().enumerate() {
        for (z, cell) in row.iter().enumerate() {
            if *cell == CellKind::Player {
                return (x as i32, z as i32);
            }
        }
    }
    (0, 0)
}

pub fn is_target(x: i32, z: i32, targets: &[(i32, i32)]) -> bool {
    targets.contains(&(x, z))
}

pub fn check_win(cells: &[Vec<CellKind>], targets: &[(i32, i32)]) -> bool {
    if targets.is_empty() { return false; }
    targets.iter().all(|&(x, z)| {
        let x = x as usize;
        let z = z as usize;
        x < cells.len() && z < cells[x].len() && cells[x][z] == CellKind::Box
    })
}