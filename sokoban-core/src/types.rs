use serde::{Deserialize, Serialize};
use std::fmt;

// ========== 方向 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// 返回 (x偏移, z偏移)
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub fn dx(self) -> i32 {
        self.delta().0
    }

    pub fn dz(self) -> i32 {
        self.delta().1
    }

    pub fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Up => write!(f, "Up"),
            Direction::Down => write!(f, "Down"),
            Direction::Left => write!(f, "Left"),
            Direction::Right => write!(f, "Right"),
        }
    }
}

// ========== 坐标 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub z: i32,
}

impl GridPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn shift(self, dir: Direction) -> Self {
        let (dx, dz) = dir.delta();
        Self {
            x: self.x + dx,
            z: self.z + dz,
        }
    }

    /// 转为世界坐标（cell_size 为每个格子的世界尺寸）
    pub fn to_world(self, cell_size: f32) -> [f32; 3] {
        [self.x as f32 * cell_size, 0.0, self.z as f32 * cell_size]
    }

    /// 从世界坐标转为网格坐标
    pub fn from_world(world: [f32; 3], cell_size: f32) -> Self {
        Self {
            x: (world[0] / cell_size).round() as i32,
            z: (world[2] / cell_size).round() as i32,
        }
    }
}

impl fmt::Display for GridPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos3D {
    pub pos: GridPos,
    pub floor: u8,
}

impl GridPos3D {
    pub fn new(x: i32, z: i32, floor: u8) -> Self {
        Self {
            pos: GridPos::new(x, z),
            floor,
        }
    }

    pub fn shift(self, dir: Direction) -> Self {
        Self {
            pos: self.pos.shift(dir),
            floor: self.floor,
        }
    }

    pub fn to_world(self, cell_size: f32, floor_height: f32) -> [f32; 3] {
        let [x, _, z] = self.pos.to_world(cell_size);
        [x, self.floor as f32 * floor_height, z]
    }
}

// ========== 颜色 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemColor {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

impl ItemColor {
    pub fn index(self) -> usize {
        match self {
            ItemColor::Red => 0,
            ItemColor::Blue => 1,
            ItemColor::Green => 2,
            ItemColor::Yellow => 3,
            ItemColor::Purple => 4,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i % 5 {
            0 => ItemColor::Red,
            1 => ItemColor::Blue,
            2 => ItemColor::Green,
            3 => ItemColor::Yellow,
            _ => ItemColor::Purple,
        }
    }

    pub fn all() -> [ItemColor; 5] {
        [
            ItemColor::Red,
            ItemColor::Blue,
            ItemColor::Green,
            ItemColor::Yellow,
            ItemColor::Purple,
        ]
    }
}

impl fmt::Display for ItemColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemColor::Red => write!(f, "Red"),
            ItemColor::Blue => write!(f, "Blue"),
            ItemColor::Green => write!(f, "Green"),
            ItemColor::Yellow => write!(f, "Yellow"),
            ItemColor::Purple => write!(f, "Purple"),
        }
    }
}

// ========== 地板类型 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloorType {
    /// 空（虚空，不可行走）
    Empty,
    /// 普通地板
    Normal,
    /// 冰面：物体沿推动方向滑行到障碍物
    Ice,
    /// 水面：人/箱子掉入重置
    Water,
    /// 深坑：同水面
    Pit,
    /// 传送带：自动推移物体
    Conveyor(Direction),
    /// 胜利目标点
    Target,
    /// 压力板：踩下触发关联机关
    PressurePlate,
    /// 传送门：配对 ID，踏入传送到另一个
    Portal(u8),
    /// 泥地：减速
    Mud,
    /// 玻璃地板：箱子推上去后碎裂变坑
    Glass,
    /// 斜坡：只能单方向通行
    Ramp(Direction),
}

impl FloorType {
    /// 是否可通行（人和箱子可以站在这上面）
    pub fn is_passable(self) -> bool {
        !matches!(self, FloorType::Empty)
    }

    /// 是否是目标点
    pub fn is_target(self) -> bool {
        matches!(self, FloorType::Target)
    }
}

impl fmt::Display for FloorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloorType::Empty => write!(f, "Empty"),
            FloorType::Normal => write!(f, "Normal"),
            FloorType::Ice => write!(f, "Ice"),
            FloorType::Water => write!(f, "Water"),
            FloorType::Pit => write!(f, "Pit"),
            FloorType::Conveyor(d) => write!(f, "Conveyor({})", d),
            FloorType::Target => write!(f, "Target"),
            FloorType::PressurePlate => write!(f, "PressurePlate"),
            FloorType::Portal(id) => write!(f, "Portal({})", id),
            FloorType::Mud => write!(f, "Mud"),
            FloorType::Glass => write!(f, "Glass"),
            FloorType::Ramp(d) => write!(f, "Ramp({})", d),
        }
    }
}

// ========== 物体类型 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    /// 空（无物体）
    None,
    /// 普通墙壁
    Wall,
    /// 裂墙：可被炸弹/脆弱箱子破坏
    CrackedWall,
    /// 标准箱子
    Box,
    /// 重型箱子：不能被普通推动
    HeavyBox,
    /// 脆弱箱子：推到裂墙旁碎裂并破坏墙
    FragileBox,
    /// 冰箱：不受传送带影响
    IceBox,
    /// 玩家
    Player,
    /// 钥匙：走过拾取，打开同色门
    Key(ItemColor),
    /// 门：拥有同色钥匙后消失
    Gate(ItemColor),
    /// 开关：踩下切换对应 ID 的机关状态
    Switch(u8),
    /// 石柱：对应 ID 开关控制升降
    Pillar(u8),
    /// 炸弹：推到裂墙旁引爆
    Bomb,
    /// 弹簧：把推过来的物体弹射
    Spring,
    /// 岩石：不可推动
    Rock,
    /// 镜子：改变推力方向 90 度
    Mirror(Direction),
    /// 磁铁
    Magnet,
    /// 尖刺：箱子推上去碎裂
    Spikes,
}

impl ObjectType {
    /// 是否是箱子类（可被推动的物体）
    pub fn is_box(self) -> bool {
        matches!(
            self,
            ObjectType::Box | ObjectType::HeavyBox | ObjectType::FragileBox | ObjectType::IceBox
        )
    }

    /// 是否是障碍物（不可通行）
    pub fn is_obstacle(self) -> bool {
        !matches!(self, ObjectType::None | ObjectType::Player)
    }

    /// 是否可被推动
    pub fn is_pushable(self) -> bool {
        matches!(
            self,
            ObjectType::Box
                | ObjectType::HeavyBox
                | ObjectType::FragileBox
                | ObjectType::IceBox
                | ObjectType::Bomb
        )
    }

    /// 是否可拾取
    pub fn is_collectible(self) -> bool {
        matches!(self, ObjectType::Key(_))
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::None => write!(f, "None"),
            ObjectType::Wall => write!(f, "Wall"),
            ObjectType::CrackedWall => write!(f, "CrackedWall"),
            ObjectType::Box => write!(f, "Box"),
            ObjectType::HeavyBox => write!(f, "HeavyBox"),
            ObjectType::FragileBox => write!(f, "FragileBox"),
            ObjectType::IceBox => write!(f, "IceBox"),
            ObjectType::Player => write!(f, "Player"),
            ObjectType::Key(c) => write!(f, "Key({})", c),
            ObjectType::Gate(c) => write!(f, "Gate({})", c),
            ObjectType::Switch(id) => write!(f, "Switch({})", id),
            ObjectType::Pillar(id) => write!(f, "Pillar({})", id),
            ObjectType::Bomb => write!(f, "Bomb"),
            ObjectType::Spring => write!(f, "Spring"),
            ObjectType::Rock => write!(f, "Rock"),
            ObjectType::Mirror(d) => write!(f, "Mirror({})", d),
            ObjectType::Magnet => write!(f, "Magnet"),
            ObjectType::Spikes => write!(f, "Spikes"),
        }
    }
}

// ========== 格子 ==========

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub floor: FloorType,
    pub object: ObjectType,
    pub color: Option<ItemColor>,
    pub facing: Option<Direction>,
    pub linked_id: Option<u8>,
}

impl Cell {
    /// 空格子（普通地板，无物体）
    pub fn empty() -> Self {
        Self {
            floor: FloorType::Normal,
            object: ObjectType::None,
            color: None,
            facing: None,
            linked_id: None,
        }
    }

    /// 墙壁格子
    pub fn wall() -> Self {
        Self {
            floor: FloorType::Empty,
            object: ObjectType::Wall,
            color: None,
            facing: None,
            linked_id: None,
        }
    }

    /// 目标点格子
    pub fn target() -> Self {
        Self {
            floor: FloorType::Target,
            object: ObjectType::None,
            color: None,
            facing: None,
            linked_id: None,
        }
    }

    /// 是否可通行（地板可走且无阻挡物体）
    pub fn is_passable(&self) -> bool {
        if !self.floor.is_passable() {
            return false;
        }
        match self.object {
            ObjectType::None | ObjectType::Player => true,
            ObjectType::Key(_) => true,
            ObjectType::Switch(_) => true,
            ObjectType::Spring => true,
            ObjectType::Spikes => true,
            ObjectType::Magnet => true,
            ObjectType::Mirror(_) => true,
            _ => false,
        }
    }

    /// 是否有可推动的物体
    pub fn has_pushable(&self) -> bool {
        self.object.is_pushable()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::empty()
    }
}

// ========== 移动相关 ==========

/// 移动意图
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveIntent {
    pub direction: Direction,
}

/// 移动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveStepType {
    /// 正常行走
    Walk,
    /// 推箱子
    Push,
    /// 冰面滑行
    Slide,
    /// 传送带
    Conveyor,
    /// 传送门
    Teleport,
    /// 弹簧弹射
    SpringBounce,
    /// 坠落（水/坑）
    Fall,
    /// 拾取物品
    Collect,
    /// 破坏物体
    Destroy,
    /// 开门
    OpenDoor,
}

/// 单步移动记录
#[derive(Debug, Clone)]
pub struct MoveStep {
    pub entity_id: u64,
    pub from: GridPos3D,
    pub to: GridPos3D,
    pub step_type: MoveStepType,
    pub floor_type: FloorType,
}

/// 移动结果
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub success: bool,
    pub steps: Vec<MoveStep>,
    pub collected_keys: Vec<ItemColor>,
    pub destroyed_entities: Vec<u64>,
    pub triggered_switches: Vec<u8>,
    pub player_died: bool,
}

impl MoveResult {
    pub fn success() -> Self {
        Self {
            success: true,
            steps: Vec::new(),
            collected_keys: Vec::new(),
            destroyed_entities: Vec::new(),
            triggered_switches: Vec::new(),
            player_died: false,
        }
    }

    pub fn failure() -> Self {
        Self {
            success: false,
            steps: Vec::new(),
            collected_keys: Vec::new(),
            destroyed_entities: Vec::new(),
            triggered_switches: Vec::new(),
            player_died: false,
        }
    }
}

// ========== 场景主题 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneTheme {
    pub id: String,
    pub name: String,
    pub environment_rules: EnvironmentRules,
    pub exclusive_mechanics: Vec<ExclusiveMechanic>,
}

impl Default for SceneTheme {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            environment_rules: EnvironmentRules::default(),
            exclusive_mechanics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentRules {
    /// 重力倍率，1.0 正常
    pub gravity_multiplier: f32,
    /// 摩擦系数，0.0 光滑 1.0 完全摩擦
    pub friction: f32,
    /// 能见度范围
    pub visibility_range: f32,
    /// 时间流速
    pub time_scale: f32,
}

impl Default for EnvironmentRules {
    fn default() -> Self {
        Self {
            gravity_multiplier: 1.0,
            friction: 0.5,
            visibility_range: 100.0,
            time_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExclusiveMechanic {
    /// 岩浆周期涨落
    LavaCycle {
        rise_interval: u32,
        retreat_interval: u32,
        pattern: Vec<(i32, i32)>,
    },
    /// 寒风周期吹动
    WindGust {
        interval: u32,
        direction: Direction,
        strength: u32,
    },
    /// 光线反射解谜
    LightBeam {
        source_pos: GridPos,
        source_dir: Direction,
        target_pos: GridPos,
    },
    /// 水位控制
    WaterLevel {
        initial_level: u8,
        max_level: u8,
    },
    /// 镜像区域
    MirrorZone {
        zone_a_origin: GridPos,
        zone_a_size: (u32, u32),
        zone_b_origin: GridPos,
        zone_b_size: (u32, u32),
    },
    /// 定时出现消失的地板
    AppearingFloor {
        positions: Vec<GridPos>,
        appear_interval: u32,
        disappear_interval: u32,
    },
    /// 天平平衡
    BalanceScale {
        left_positions: Vec<GridPos>,
        right_positions: Vec<GridPos>,
        max_weight_diff: u32,
        linked_gate: u8,
    },
}

// ========== 地牢相关 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    Puzzle,
    Treasure,
    Shop,
    Challenge,
    Boss,
    Stairwell,
    Rest,
}

impl Default for RoomType {
    fn default() -> Self {
        RoomType::Puzzle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DungeonItem {
    /// 一次性炸弹
    BombPickup,
    /// 翅膀：跳过一格
    Wing,
    /// 手套：拉回箱子
    Glove,
    /// 传送器：标记位置后传送回去
    Teleporter,
    /// 护盾：箱子落水不消失
    Shield,
}

impl fmt::Display for DungeonItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DungeonItem::BombPickup => write!(f, "Bomb"),
            DungeonItem::Wing => write!(f, "Wing"),
            DungeonItem::Glove => write!(f, "Glove"),
            DungeonItem::Teleporter => write!(f, "Teleporter"),
            DungeonItem::Shield => write!(f, "Shield"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardType {
    UnlockKey(String),
    Item(DungeonItem),
    RevealMap,
    ExtraUndo,
    HintToken,
}

// ========== 跨层连接 ==========

/// 跨楼层连接（用于运行时检测）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloorLink {
    pub from_pos: GridPos,
    pub from_floor: u8,
    pub to_pos: GridPos,
    pub to_floor: u8,
    pub link_type: FloorLinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorLinkType {
    Stairs,
    Ladder,
    Elevator,
    Hole,
    Portal,
}

// ========== 死局类型 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlockType {
    /// 角落死锁：箱子在墙角
    Corner,
    /// 边缘死锁：箱子在隧道中且该边无目标点
    Edge,
    /// 冻结死锁：箱子互相卡住
    Frozen,
}

// ========== 验证结果 ==========

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            issues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValidationIssue {
    NoBoxes,
    NoTargets,
    BoxTargetMismatch { boxes: u32, targets: u32 },
    NoPlayerSpawn,
    MultiplePlayerSpawns { count: u32 },
    NotConnected,
    NoFloors,
    NoRooms,
    MissingKey { color: ItemColor },
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationIssue::NoBoxes => write!(f, "No boxes placed"),
            ValidationIssue::NoTargets => write!(f, "No targets placed"),
            ValidationIssue::BoxTargetMismatch { boxes, targets } => {
                write!(f, "Box count ({}) != target count ({})", boxes, targets)
            }
            ValidationIssue::NoPlayerSpawn => write!(f, "No player spawn point"),
            ValidationIssue::MultiplePlayerSpawns { count } => {
                write!(f, "Multiple player spawns ({})", count)
            }
            ValidationIssue::NotConnected => write!(f, "Map is not fully connected"),
            ValidationIssue::NoFloors => write!(f, "No floor layers defined"),
            ValidationIssue::NoRooms => write!(f, "No rooms defined"),
            ValidationIssue::MissingKey { color } => {
                write!(f, "Gate requires key {:?} but none placed", color)
            }
        }
    }
}

// ========== 难度评估 ==========

#[derive(Debug, Clone)]
pub struct DifficultyReport {
    pub score: f32,
    pub star_rating: u8,
    pub optimal_steps: Option<u32>,
    pub box_count: u32,
    pub item_count: u32,
    pub has_dead_ends: bool,
    pub branching_factor: f32,
}


