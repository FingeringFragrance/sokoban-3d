use std::collections::HashMap;

use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::types::*;

use bevy::prelude::Resource;

#[allow(dead_code)]
pub const FLOOR_HEIGHT: f32 = 4.0;

#[derive(Resource)]
pub struct MultiFloorRun {
    pub level: Option<MultiFloorLevel>,
    pub current_floor: u8,
    pub saved_states: HashMap<u8, GridState>,
    pub active: bool,
}

impl Default for MultiFloorRun {
    fn default() -> Self {
        Self {
            level: None,
            current_floor: 0,
            saved_states: HashMap::new(),
            active: false,
        }
    }
}

impl MultiFloorRun {
    pub fn load_demo(&mut self) {
        self.level = Some(create_demo_multifloor());
        self.current_floor = 0;
        self.saved_states.clear();
        self.active = true;
    }

    pub fn current_floor_data(&self) -> Option<&FloorLayer> {
        self.level
            .as_ref()
            .and_then(|l| l.get_floor(self.current_floor))
    }

    pub fn floor_count(&self) -> u8 {
        self.level.as_ref().map(|l| l.floor_count()).unwrap_or(0)
    }

    pub fn current_elevation(&self) -> f32 {
        self.current_floor_data()
            .map(|f| f.elevation)
            .unwrap_or(0.0)
    }

    #[allow(dead_code)]
    pub fn all_floors_complete(&self, current_grid: &GridState) -> bool {
        if !current_grid.all_boxes_on_targets() {
            return false;
        }
        for grid in self.saved_states.values() {
            if !grid.all_boxes_on_targets() {
                return false;
            }
        }
        true
    }
}

fn create_demo_multifloor() -> MultiFloorLevel {
    MultiFloorLevel {
        meta: LevelMeta {
            id: 100,
            name: "Twin Towers".to_string(),
            author: "Designer".to_string(),
            difficulty: 3,
            par_steps: Some(25),
            tags: vec!["multifloor".to_string()],
            description: "Solve puzzles across two floors. Press 1/2 to switch.".to_string(),
        },
        scene_theme: "default".to_string(),
        floors: vec![
            FloorLayer {
                level: 0,
                grid: Grid::from_ascii(&[
                    "#########",
                    "#.......#",
                    "#.@..$..#",
                    "#.......#",
                    "#...x...#",
                    "#.......#",
                    "#########",
                ]),
                elevation: 0.0,
            },
            FloorLayer {
                level: 1,
                grid: Grid::from_ascii(&[
                    "#########",
                    "#.......#",
                    "#.$.$...#",
                    "#..@....#",
                    "#..xx...#",
                    "#.......#",
                    "#########",
                ]),
                elevation: FLOOR_HEIGHT,
            },
        ],
        connections: vec![
            FloorConnection {
                connection_type: ConnectionType::Stairs(Direction::Right),
                from_floor: 0,
                from_pos: GridPos::new(7, 3),
                to_floor: 1,
                to_pos: GridPos::new(1, 3),
            },
            FloorConnection {
                connection_type: ConnectionType::Stairs(Direction::Left),
                from_floor: 1,
                from_pos: GridPos::new(1, 3),
                to_floor: 0,
                to_pos: GridPos::new(7, 3),
            },
        ],
    }
}
