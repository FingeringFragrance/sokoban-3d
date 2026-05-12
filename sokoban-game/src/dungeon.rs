use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DungeonItemType {
    Bomb,
    Wing,
    Glove,
    Teleporter,
    Shield,
}

impl DungeonItemType {
    pub fn label(&self) -> &'static str {
        match self {
            DungeonItemType::Bomb => "Bomb",
            DungeonItemType::Wing => "Wing",
            DungeonItemType::Glove => "Glove",
            DungeonItemType::Teleporter => "Teleporter",
            DungeonItemType::Shield => "Shield",
        }
    }

    pub fn key_hint(&self) -> &'static str {
        match self {
            DungeonItemType::Bomb => "1",
            DungeonItemType::Wing => "2",
            DungeonItemType::Glove => "3",
            DungeonItemType::Teleporter => "4",
            DungeonItemType::Shield => "5",
        }
    }
}

#[derive(Resource)]
pub struct DungeonManager {
    pub dungeon: Option<DungeonData>,
    pub room_order: Vec<String>,
    pub current_index: usize,
    pub active: bool,
    pub inventory: Vec<DungeonItemType>,
    pub max_inventory: usize,
    pub explored: HashSet<usize>,
    pub completed_rooms: HashSet<usize>,
    pub extra_undos: u32,
    pub hint_tokens: u32,
    pub teleporter_pos: Option<GridPos>,
    pub teleporter_placed: bool,
    pub shield_active: bool,
}

impl Default for DungeonManager {
    fn default() -> Self {
        Self {
            dungeon: None,
            room_order: Vec::new(),
            current_index: 0,
            active: false,
            inventory: Vec::new(),
            max_inventory: 5,
            explored: HashSet::new(),
            completed_rooms: HashSet::new(),
            extra_undos: 0,
            hint_tokens: 0,
            teleporter_pos: None,
            teleporter_placed: false,
            shield_active: false,
        }
    }
}

impl DungeonManager {
    pub fn load_demo(&mut self) {
        let dungeon = create_demo_dungeon();
        let order = build_room_order(&dungeon);
        self.dungeon = Some(dungeon);
        self.room_order = order;
        self.current_index = 0;
        self.active = true;
        self.inventory.clear();
        self.explored.clear();
        self.completed_rooms.clear();
        self.extra_undos = 0;
        self.hint_tokens = 0;
        self.teleporter_pos = None;
        self.teleporter_placed = false;
        self.shield_active = false;
        self.explored.insert(0);
    }

    pub fn current_room_grid(&self) -> Option<&Grid> {
        self.dungeon.as_ref().and_then(|d| {
            self.room_order
                .get(self.current_index)
                .and_then(|id| d.get_room(id))
                .map(|room| &room.grid)
        })
    }

    pub fn current_room_name(&self) -> String {
        self.dungeon
            .as_ref()
            .and_then(|d| {
                self.room_order
                    .get(self.current_index)
                    .and_then(|id| d.get_room(id))
                    .map(|room| {
                        format!("{} ({})", room.room_id, format_room_type(room.room_type))
                    })
            })
            .unwrap_or_default()
    }

    pub fn advance(&mut self) -> bool {
        self.completed_rooms.insert(self.current_index);
        self.collect_reward();
        self.current_index += 1;
        if self.current_index < self.room_order.len() {
            self.explored.insert(self.current_index);
            true
        } else {
            false
        }
    }

    pub fn room_count(&self) -> usize {
        self.room_order.len()
    }

    pub fn has_item(&self, item: DungeonItemType) -> bool {
        self.inventory.contains(&item)
    }

    pub fn use_item(&mut self, item: DungeonItemType) -> bool {
        if let Some(pos) = self.inventory.iter().position(|&i| i == item) {
            self.inventory.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn add_item(&mut self, item: DungeonItemType) {
        if self.inventory.len() < self.max_inventory {
            self.inventory.push(item);
        }
    }

    fn collect_reward(&mut self) {
        let reward = self
            .dungeon
            .as_ref()
            .and_then(|d| {
                self.room_order
                    .get(self.current_index)
                    .and_then(|id| d.get_room(id))
                    .and_then(|room| room.reward.as_ref())
            });

        match reward {
            Some(RewardType::Item(dungeon_item)) => {
                let item = match dungeon_item {
                    DungeonItem::BombPickup => DungeonItemType::Bomb,
                    DungeonItem::Wing => DungeonItemType::Wing,
                    DungeonItem::Glove => DungeonItemType::Glove,
                    DungeonItem::Teleporter => DungeonItemType::Teleporter,
                    DungeonItem::Shield => DungeonItemType::Shield,
                };
                self.add_item(item);
            }
            Some(RewardType::ExtraUndo) => {
                self.extra_undos += 5;
            }
            Some(RewardType::HintToken) => {
                self.hint_tokens += 1;
            }
            _ => {}
        }
    }

    pub fn minimap_data(&self) -> Vec<(usize, String, RoomType, bool, bool, bool)> {
        let mut result = Vec::new();
        for (i, room_id) in self.room_order.iter().enumerate() {
            let (name, rt) = self
                .dungeon
                .as_ref()
                .and_then(|d| d.get_room(room_id))
                .map(|r| (r.room_id.clone(), r.room_type))
                .unwrap_or_else(|| (room_id.clone(), RoomType::Puzzle));

            result.push((
                i,
                name,
                rt,
                self.explored.contains(&i),
                self.completed_rooms.contains(&i),
                i == self.current_index,
            ));
        }
        result
    }
}

fn format_room_type(rt: RoomType) -> &'static str {
    match rt {
        RoomType::Puzzle => "Puzzle",
        RoomType::Treasure => "Treasure",
        RoomType::Shop => "Shop",
        RoomType::Challenge => "Challenge",
        RoomType::Boss => "Boss",
        RoomType::Stairwell => "Stairwell",
        RoomType::Rest => "Rest",
    }
}

fn build_room_order(dungeon: &DungeonData) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut order = Vec::new();

    visited.insert(dungeon.start_room.clone());
    queue.push_back(dungeon.start_room.clone());

    while let Some(current) = queue.pop_front() {
        order.push(current.clone());
        if let Some(room) = dungeon.rooms.get(&current) {
            for conn in &room.connections {
                if !conn.is_locked && !visited.contains(&conn.target_room) {
                    visited.insert(conn.target_room.clone());
                    queue.push_back(conn.target_room.clone());
                }
            }
        }
    }

    order
}

fn create_demo_dungeon() -> DungeonData {
    let room1 = RoomSlot {
        room_id: "Entrance".to_string(),
        room_type: RoomType::Puzzle,
        connections: vec![RoomConnection {
            direction: Direction::Right,
            target_room: "Armory".to_string(),
            is_locked: false,
            lock_color: None,
        }],
        floor_level: 0,
        reward: None,
        grid: Grid::from_ascii(&[
            "#######",
            "#.....#",
            "#.@$..#",
            "#.....#",
            "#..x..#",
            "#######",
        ]),
    };

    let room2 = RoomSlot {
        room_id: "Armory".to_string(),
        room_type: RoomType::Treasure,
        connections: vec![RoomConnection {
            direction: Direction::Right,
            target_room: "KeyHall".to_string(),
            is_locked: false,
            lock_color: None,
        }],
        floor_level: 0,
        reward: Some(RewardType::Item(DungeonItem::Glove)),
        grid: Grid::from_ascii(&[
            "########",
            "#......#",
            "#.@k$..#",
            "#..#...#",
            "#....$.#",
            "#......#",
            "#.x..x.#",
            "########",
        ]),
    };

    let room3 = RoomSlot {
        room_id: "KeyHall".to_string(),
        room_type: RoomType::Puzzle,
        connections: vec![
            RoomConnection {
                direction: Direction::Right,
                target_room: "Shop".to_string(),
                is_locked: false,
                lock_color: None,
            },
            RoomConnection {
                direction: Direction::Down,
                target_room: "Vault".to_string(),
                is_locked: false,
                lock_color: None,
            },
        ],
        floor_level: 0,
        reward: None,
        grid: Grid::from_ascii(&[
            "##########",
            "#........#",
            "#.k.@.$..#",
            "#...##...#",
            "#.D......#",
            "#...#..$..#",
            "#........#",
            "#..x...x.#",
            "#........#",
            "##########",
        ]),
    };

    let room4 = RoomSlot {
        room_id: "Shop".to_string(),
        room_type: RoomType::Shop,
        connections: vec![RoomConnection {
            direction: Direction::Right,
            target_room: "Arena".to_string(),
            is_locked: false,
            lock_color: None,
        }],
        floor_level: 0,
        reward: Some(RewardType::ExtraUndo),
        grid: Grid::from_ascii(&[
            "#######",
            "#.....#",
            "#.@...#",
            "#.....#",
            "#.....#",
            "#######",
        ]),
    };

    let room5 = RoomSlot {
        room_id: "Arena".to_string(),
        room_type: RoomType::Challenge,
        connections: vec![RoomConnection {
            direction: Direction::Right,
            target_room: "Throne".to_string(),
            is_locked: false,
            lock_color: None,
        }],
        floor_level: 0,
        reward: Some(RewardType::Item(DungeonItem::BombPickup)),
        grid: Grid::from_ascii(&[
            "##########",
            "#........#",
            "#.@..$...#",
            "#...##...#",
            "#..$.....#",
            "#........#",
            "#.....$..#",
            "#..x.x.x.#",
            "#........#",
            "##########",
        ]),
    };

    let room6 = RoomSlot {
        room_id: "Vault".to_string(),
        room_type: RoomType::Treasure,
        connections: vec![],
        floor_level: 0,
        reward: Some(RewardType::HintToken),
        grid: Grid::from_ascii(&[
            "########",
            "#......#",
            "#.@.$..#",
            "#..#...#",
            "#....$.#",
            "#......#",
            "#.x..x.#",
            "########",
        ]),
    };

    let room7 = RoomSlot {
        room_id: "Throne".to_string(),
        room_type: RoomType::Boss,
        connections: vec![],
        floor_level: 0,
        reward: Some(RewardType::RevealMap),
        grid: Grid::from_ascii(&[
            "############",
            "#..........#",
            "#.@..$.....#",
            "#....##....#",
            "#..$.......#",
            "#..........#",
            "#.......$..#",
            "#..........#",
            "#.x..x..x..#",
            "#..........#",
            "############",
        ]),
    };

    let mut rooms = HashMap::new();
    rooms.insert("Entrance".to_string(), room1);
    rooms.insert("Armory".to_string(), room2);
    rooms.insert("KeyHall".to_string(), room3);
    rooms.insert("Shop".to_string(), room4);
    rooms.insert("Arena".to_string(), room5);
    rooms.insert("Vault".to_string(), room6);
    rooms.insert("Throne".to_string(), room7);

    DungeonData {
        id: 1,
        name: "Demo Dungeon".to_string(),
        theme: "cave".to_string(),
        rooms,
        start_room: "Entrance".to_string(),
        boss_room: "Throne".to_string(),
    }
}
