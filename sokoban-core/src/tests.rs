use crate::grid::*;
use crate::history::*;
use crate::level::*;
use crate::replay::*;
use crate::rules::*;
use crate::solver::*;
use crate::types::*;

// ============================================================
//  辅助函数
// ============================================================

/// 构建关卡
fn make_level(w: usize, h: usize, layout: &[&str]) -> Grid {
    let mut cells = vec![vec![Cell::empty(); w]; h];
    for (z, row) in layout.iter().enumerate() {
        let chars: Vec<char> = row.chars().filter(|c| !c.is_whitespace()).collect();
        for (x, ch) in chars.iter().enumerate() {
            if x >= w || z >= h {
                continue;
            }
            match ch {
                '#' => cells[z][x] = Cell::wall(),
                '.' => cells[z][x] = Cell::empty(),
                '@' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Player,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                '$' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Box,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'x' => cells[z][x] = Cell::target(),
                '~' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Ice,
                        object: ObjectType::None,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'k' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Key(ItemColor::Red),
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'D' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Gate(ItemColor::Red),
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'H' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::HeavyBox,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'F' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::FragileBox,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'i' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::IceBox,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'B' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Bomb,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'S' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Spring,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'r' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Rock,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'c' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::CrackedWall,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'W' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Water,
                        object: ObjectType::None,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'O' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Pit,
                        object: ObjectType::None,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'M' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Mud,
                        object: ObjectType::None,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                'G' => {
                    cells[z][x] = Cell {
                        floor: FloorType::Glass,
                        object: ObjectType::None,
                        color: None,
                        facing: None,
                        linked_id: None,
                    };
                }
                _ => {}
            }
        }
    }
    Grid {
        width: w as u32,
        height: h as u32,
        cells,
    }
}

fn default_scene() -> SceneTheme {
    SceneTheme::default()
}

// ============================================================
//  基础移动测试
// ============================================================

/// 地图：
///   # # # # #
///   # . @ . #
///   # . . . #
///   # # # # #
#[test]
fn test_player_move_up() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# . @ . #",
        "# . . . #",
        "# # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，上边 (2,0) 是墙
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Up }, &scene);
    assert!(!result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 1));

    // 往下走到 (2,2)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Down }, &scene);
    assert!(result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 2));
    assert_eq!(state.current_step, 1);
}

#[test]
fn test_player_move_into_wall() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# . @ . #",
        "# . . . #",
        "# # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，右边 (3,1) 是空地
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(3, 1));

    // 再往右 (4,1) 是墙
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(!result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(3, 1));
}

/// 地图：
///   # # # # #
///   # . @ . #
///   # $ . . #
///   # . . . #
///   # # # # #
#[test]
fn test_player_move_into_empty() {
    let grid = make_level(5, 5, &[
        "# # # # #",
        "# . @ . #",
        "# $ . . #",
        "# . . . #",
        "# # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，往下走到 (2,2) 空地
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Down }, &scene);
    assert!(result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 2));

    // 往左走到 (1,2)，那里有箱子，推到 (0,2) 撞墙
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Left }, &scene);
    assert!(!result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 2));
}

// ============================================================
//  推箱子测试
// ============================================================

/// 地图：
///   # # # # # #
///   # . . . . #
///   # . @ $ . #
///   # . . x . #
///   # . . . . #
///   # # # # # #
#[test]
fn test_push_box_left() {
    let grid = make_level(6, 6, &[
        "# # # # # #",
        "# . . . . #",
        "# . @ $ . #",
        "# . . x . #",
        "# . . . . #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,2)，箱子在 (3,2)
    // 往右推：玩家到 (3,2)，箱子到 (4,2)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(3, 2));
    assert!(state.find_box_at(GridPos::new(4, 2)).is_some());
    assert!(state.find_box_at(GridPos::new(3, 2)).is_none());
}

#[test]
fn test_push_box_into_wall() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# . @ $ #",
        "# . . . #",
        "# # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，箱子在 (3,1)，右边是墙 (4,1)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(!result.success);
    assert!(state.find_box_at(GridPos::new(3, 1)).is_some());
}

/// 地图：
///   # # # # # #
///   # . . . . #
///   # . @ $ . #
///   # . . x . #
///   # . . . . #
///   # # # # # #
#[test]
fn test_push_box_onto_target() {
    let grid = make_level(6, 6, &[
        "# # # # # #",
        "# . . . . #",
        "# . @ $ . #",
        "# . . x . #",
        "# . . . . #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 箱子在 (3,2)，目标点在 (3,3)
    // 需要把箱子往下推：玩家先到 (3,1)
    // 往上到 (2,1)
    resolve_move(&mut state, MoveIntent { direction: Direction::Up }, &scene);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 1));
    // 往右到 (3,1)
    resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert_eq!(state.player_pos.pos, GridPos::new(3, 1));
    // 往下推箱子：玩家到 (3,2)，箱子到 (3,3)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Down }, &scene);
    assert!(result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(3, 2));
    assert!(state.find_box_at(GridPos::new(3, 3)).is_some());
    assert!(state.is_target(GridPos::new(3, 3)));
    assert_eq!(state.boxes_on_targets(), 1);
}

/// 两个箱子并排推不动
///   # # # # # #
///   # @ $ $ . #
///   # . . . . #
///   # # # # # #
#[test]
fn test_cannot_push_two_boxes() {
    let grid = make_level(6, 4, &[
        "# # # # # #",
        "# @ $ $ . #",
        "# . . . . #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (1,1)，箱子在 (2,1) 和 (3,1)
    // 推左边箱子，前面还有一个箱子，推不动
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(!result.success);
}

// ============================================================
//  钥匙和门测试
// ============================================================

/// 地图：
///   # # # # # #
///   # @ k D . #
///   # . . . . #
///   # # # # # #
#[test]
fn test_key_opens_door() {
    let grid = make_level(6, 4, &[
        "# # # # # #",
        "# @ k D . #",
        "# . . . . #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (1,1)，钥匙在 (2,1)，门在 (3,1)

    // 没有钥匙时门挡住
    // 先直接试走门：不行，因为钥匙在 (2,1) 挡不住...
    // 钥匙是可通行的，所以先走到 (2,1) 会拾取钥匙
    // 改为：先走到 (1,2) 绕过钥匙
    resolve_move(&mut state, MoveIntent { direction: Direction::Down }, &scene);
    assert_eq!(state.player_pos.pos, GridPos::new(1, 2));
    assert!(!state.has_key(ItemColor::Red));

    // 走到 (3,2)
    resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    // 走到 (3,1) 门位置，没有钥匙应该被挡
    let r = resolve_move(&mut state, MoveIntent { direction: Direction::Up }, &scene);
    assert!(!r.success);

    // 重新来，走钥匙路线
    let mut state = GridState::from_grid(&grid, 0);

    // 右走到 (2,1) 拾取钥匙
    let r = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(r.success);
    assert!(state.has_key(ItemColor::Red));

    // 右走到 (3,1) 门位置，有钥匙开门
    let r = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(r.success);
    assert_eq!(state.object_at(GridPos::new(3, 1)), ObjectType::None);

    // 继续往右到 (4,1)
    let r = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(r.success);
    assert_eq!(state.player_pos.pos, GridPos::new(4, 1));
}

/// 没有钥匙时门阻挡
///   # # # # #
///   # @ D . #
///   # . . . #
///   # # # # #
#[test]
fn test_door_blocks_without_key() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# @ D . #",
        "# . . . #",
        "# # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (1,1)，门在 (2,1)，没有钥匙
    let r = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(!r.success);
    assert_eq!(state.player_pos.pos, GridPos::new(1, 1));
}

// ============================================================
//  冰面测试
// ============================================================

/// 地图：
///   # # # # # #
///   # @ . $ ~ #
///   # . . . x #
///   # # # # # #
#[test]
fn test_ice_slide() {
    let grid = make_level(6, 4, &[
        "# # # # # #",
        "# @ . $ ~ #",
        "# . . . x #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 箱子在 (3,1)，冰面在 (4,1)，墙在 (5,1)
    // 玩家在 (1,1)
    // 走到 (2,1)
    resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    // 推箱子：玩家到 (3,1)，箱子到 (4,1) 冰面
    // 冰面继续沿 Right 滑行，(5,1) 是墙，停在 (4,1)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(result.success);
    assert!(state.find_box_at(GridPos::new(4, 1)).is_some());
}

// ============================================================
//  撤销测试
// ============================================================

#[test]
fn test_undo() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# . @ . #",
        "# . . . #",
        "# # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let mut history = MoveHistory::new();
    let scene = default_scene();

    let initial_pos = state.player_pos;

    // 保存快照并移动
    history.push(state.snapshot(), Direction::Down);
    resolve_move(&mut state, MoveIntent { direction: Direction::Down }, &scene);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 2));

    // 撤销
    let snapshot = history.pop().unwrap();
    state.restore(&snapshot);
    assert_eq!(state.player_pos, initial_pos);
    assert_eq!(state.current_step, 0);
}

#[test]
fn test_undo_empty_history() {
    let mut history = MoveHistory::new();
    assert!(history.pop().is_none());
    assert!(history.is_empty());
}

// ============================================================
//  胜利条件测试
// ============================================================

/// 地图：
///   # # # # # #
///   # . . . . #
///   # . @ $ . #
///   # . x . . #
///   # . . . . #
///   # # # # # #
#[test]
fn test_win_condition() {
    let grid = make_level(6, 6, &[
        "# # # # # #",
        "# . . . . #",
        "# . @ $ . #",
        "# . x . . #",
        "# . . . . #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    assert!(!state.all_boxes_on_targets());

    // 箱子在 (3,2)，目标点在 (2,3)
    // 路线：上、右、下推箱子到(3,3)，左、下、左、上、左推箱子到(2,3)
    // 简化：直接手动设置验证逻辑
    let mut state2 = GridState::from_grid(&grid, 0);
    state2.box_positions[0].pos = GridPos3D::new(2, 3, 0);
    assert!(state2.all_boxes_on_targets());

    // 测试不是全部到位
    state2.box_positions[0].pos = GridPos3D::new(3, 2, 0);
    assert!(!state2.all_boxes_on_targets());
}

// ============================================================
//  死局检测测试
// ============================================================

/// 箱子在角落 (1,1)，左和上都是墙
///   # # # #
///   # $ . #
///   # . x #
///   # # # #
#[test]
fn test_corner_deadlock() {
    let grid = make_level(4, 4, &[
        "# # # #",
        "# $ . #",
        "# . x #",
        "# # # #",
    ]);
    let state = GridState::from_grid(&grid, 0);

    let deadlock = detect_deadlock(&state);
    assert!(deadlock.is_some());
    assert_eq!(deadlock.unwrap(), DeadlockType::Corner);
}

// ============================================================
//  求解器测试
// ============================================================

/// 地图：
///   # # # # #
///   # @ . . #
///   # . $ . #
///   # . x . #
///   # # # # #
///
/// 解法：Right, Down（玩家到(2,1)然后推箱子到(2,3)）
#[test]
fn test_solver_simple() {
    let grid = make_level(5, 5, &[
        "# # # # #",
        "# @ . . #",
        "# . $ . #",
        "# . x . #",
        "# # # # #",
    ]);
    let state = GridState::from_grid(&grid, 0);

    let config = SolverConfig {
        max_states: 10_000,
        timeout_ms: 3_000,
    };
    let result = solve(&state, &config);

    assert!(result.solution.is_some());
    let solution = result.solution.unwrap();
    // 最短解法应该很短
    assert!(solution.len() <= 10);
    assert!(solution.len() >= 2);
}

// ============================================================
//  回放测试
// ============================================================

#[test]
fn test_replay_encode_decode() {
    let mut replay = ReplayData::new(1, "abc123".to_string());
    replay.record_move(Direction::Up);
    replay.record_move(Direction::Right);
    replay.record_move(Direction::Down);
    replay.record_move(Direction::Left);
    replay.record_move(Direction::Up);
    replay.record_move(Direction::Up);

    let encoded = replay.encode();
    let decoded = ReplayData::decode(&encoded).unwrap();

    assert_eq!(decoded.level_id, 1);
    assert_eq!(decoded.total_steps, 6);
    assert_eq!(decoded.moves.len(), 6);
    assert_eq!(decoded.moves[0], Direction::Up);
    assert_eq!(decoded.moves[1], Direction::Right);
    assert_eq!(decoded.moves[2], Direction::Down);
    assert_eq!(decoded.moves[3], Direction::Left);
    assert_eq!(decoded.moves[4], Direction::Up);
    assert_eq!(decoded.moves[5], Direction::Up);
}

#[test]
fn test_replay_player() {
    let mut replay = ReplayData::new(1, "abc".to_string());
    replay.record_move(Direction::Right);
    replay.record_move(Direction::Down);

    let mut player = ReplayPlayer::new(replay);
    assert!(!player.is_playing);
    assert_eq!(player.progress(), 0.0);

    player.play();
    assert!(player.is_playing);

    let m1 = player.next_move();
    assert_eq!(m1, Some(Direction::Right));
    assert_eq!(player.progress(), 0.5);
    assert!(player.is_playing);

    let m2 = player.next_move();
    assert_eq!(m2, Some(Direction::Down));
    assert!(player.is_finished());
    assert!(!player.is_playing);
}

// ============================================================
//  关卡验证测试
// ============================================================

/// 1 个箱子 2 个目标点 → 不匹配
#[test]
fn test_validate_valid_level() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# . $ @ #",
        "# . x x #",
        "# # # # #",
    ]);
    let level = LevelData {
        meta: LevelMeta::default(),
        grid: Some(grid),
        ascii: None,
        scene_theme: "default".to_string(),
    };

    let result = validate_level(&level);
    assert!(!result.is_valid);
    assert!(result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::BoxTargetMismatch { .. })));
}

/// 没有玩家出生点
#[test]
fn test_validate_no_player() {
    let grid = make_level(4, 4, &[
        "# # # #",
        "# $ x #",
        "# . . #",
        "# # # #",
    ]);
    let level = LevelData {
        meta: LevelMeta::default(),
        grid: Some(grid),
        ascii: None,
        scene_theme: "default".to_string(),
    };

    let result = validate_level(&level);
    assert!(!result.is_valid);
    assert!(result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::NoPlayerSpawn)));
}

// ============================================================
//  ASCII 输出测试
// ============================================================

#[test]
fn test_ascii_output() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# . $ @ #",
        "# . x x #",
        "# # # # #",
    ]);
    let state = GridState::from_grid(&grid, 0);
    let ascii = state.to_ascii();

    // 验证关键字符存在
    assert!(ascii.contains('@'));
    assert!(ascii.contains('$'));
    assert!(ascii.contains('x'));

    // 验证行列数正确（4 行，每行末尾有换行）
    let lines: Vec<&str> = ascii.trim().lines().collect();
    assert_eq!(lines.len(), 4);
}

// ============================================================
//  重型箱子测试
// ============================================================

/// 重型箱子不可推动
///   # # # # # #
///   # . @ H . #
///   # . . . . #
///   # # # # # #
#[test]
fn test_heavy_box_cannot_push() {
    let grid = make_level(6, 4, &[
        "# # # # # #",
        "# . @ H . #",
        "# . . . . #",
        "# # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，重型箱子在 (3,1)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(!result.success);
    assert_eq!(state.player_pos.pos, GridPos::new(2, 1));
    assert!(state.find_box_at(GridPos::new(3, 1)).is_some());
}

// ============================================================
//  脆弱箱子 / 炸弹撞裂墙测试
// ============================================================

/// 脆弱箱子推入裂墙，同归于尽
///   # # # # # # #
///   # . @ F c . #
///   # . . . . . #
///   # # # # # # #
#[test]
fn test_fragile_box_destroys_cracked_wall() {
    let grid = make_level(7, 4, &[
        "# # # # # # #",
        "# . @ F c . #",
        "# . . . . . #",
        "# # # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，脆弱箱子在 (3,1)，裂墙在 (4,1)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(result.success);
    // 箱子和裂墙都应该被销毁
    assert!(state.find_box_at(GridPos::new(3, 1)).is_none());
    assert!(state.find_box_at(GridPos::new(4, 1)).is_none());
    assert_eq!(state.object_at(GridPos::new(4, 1)), ObjectType::None);
    // 玩家应该移动到箱子原来的位置
    assert_eq!(state.player_pos.pos, GridPos::new(3, 1));
    assert_eq!(result.destroyed_entities.len(), 1);
}

/// 炸弹推入裂墙，同归于尽
///   # # # # # # #
///   # . @ B c . #
///   # . . . . . #
///   # # # # # # #
#[test]
fn test_bomb_destroys_cracked_wall() {
    let grid = make_level(7, 4, &[
        "# # # # # # #",
        "# . @ B c . #",
        "# . . . . . #",
        "# # # # # # #",
    ]);
    let mut state = GridState::from_grid(&grid, 0);
    let scene = default_scene();

    // 玩家在 (2,1)，炸弹在 (3,1)，裂墙在 (4,1)
    let result = resolve_move(&mut state, MoveIntent { direction: Direction::Right }, &scene);
    assert!(result.success);
    assert!(state.find_box_at(GridPos::new(3, 1)).is_none());
    assert!(state.find_box_at(GridPos::new(4, 1)).is_none());
    assert_eq!(state.object_at(GridPos::new(4, 1)), ObjectType::None);
    assert_eq!(state.player_pos.pos, GridPos::new(3, 1));
    assert_eq!(result.destroyed_entities.len(), 1);
}

// ============================================================
//  隧道死锁测试
// ============================================================

/// 箱子在垂直隧道中（左右均被墙堵住），且该列无目标点 → 隧道死锁
///   # # # # # # #
///   # . . . . . #
///   # # $ # . . #
///   # . . . . . #
///   # . . . x . #
///   # # # # # # #
#[test]
fn test_tunnel_deadlock_vertical() {
    let grid = make_level(7, 6, &[
        "# # # # # # #",
        "# . . . . . #",
        "# # $ # . . #",
        "# . . . . . #",
        "# . . . x . #",
        "# # # # # # #",
    ]);
    let state = GridState::from_grid(&grid, 0);

    let deadlock = detect_deadlock(&state);
    assert!(deadlock.is_some());
    assert_eq!(deadlock.unwrap(), DeadlockType::Edge);
}

/// 箱子仅一侧靠墙，另一侧可推 → 不是死锁
///   # # # # # #
///   # . . . . #
///   # # $ . . #
///   # . . . . #
///   # . . . x #
///   # # # # # #
#[test]
fn test_no_false_positive_edge_deadlock() {
    let grid = make_level(6, 6, &[
        "# # # # # #",
        "# . . . . #",
        "# # $ . . #",
        "# . . . . #",
        "# . . . x #",
        "# # # # # #",
    ]);
    let state = GridState::from_grid(&grid, 0);   // 去掉 mut

    let deadlock = detect_deadlock(&state);
    // 仅左侧靠墙，右侧可推，不应判定为死锁
    assert!(deadlock.is_none());
}

// ============================================================
//  求解器特殊箱子测试
// ============================================================

/// 求解器应能正确处理含重型箱子的关卡（重型箱子不可推，等同于障碍物）
///   # # # # # #
///   # @ . . . #
///   # . $ . . #
///   # H x . . #
///   # # # # # #
#[test]
fn test_solver_with_heavy_box() {
    let grid = make_level(5, 5, &[
        "# # # # #",
        "# @ . . #",
        "# . $ . #",
        "# H x . #",
        "# # # # #",
    ]);
    let state = GridState::from_grid(&grid, 0);

    let config = SolverConfig {
        max_states: 10_000,
        timeout_ms: 3_000,
    };
    let result = solve(&state, &config);

    // 普通箱子可以推到目标点，重型箱子不影响
    assert!(result.solution.is_some());
}

// ============================================================
//  验证测试补充
// ============================================================

/// 关卡有门无对应钥匙 → 验证失败
#[test]
fn test_validate_missing_key() {
    let grid = make_level(5, 4, &[
        "# # # # #",
        "# @ D x #",
        "# . $ . #",
        "# # # # #",
    ]);
    let level = LevelData {
        meta: LevelMeta::default(),
        grid: Some(grid),
        ascii: None,
        scene_theme: "default".to_string(),
    };

    let result = validate_level(&level);
    assert!(!result.is_valid);
    assert!(result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::MissingKey { .. })));
}

/// 关卡有门且有对应钥匙 → 无 MissingKey 问题
#[test]
fn test_validate_key_door_pair() {
    let grid = make_level(6, 4, &[
        "# # # # # #",
        "# @ k D x #",
        "# . $ . . #",
        "# # # # # #",
    ]);
    let level = LevelData {
        meta: LevelMeta::default(),
        grid: Some(grid),
        ascii: None,
        scene_theme: "default".to_string(),
    };

    let result = validate_level(&level);
    // 1 箱 1 目标，有玩家，有钥匙配对门，应通过（或仅报告其他非致命问题）
    assert!(!result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::MissingKey { .. })));
}

// ============================================================
//  回放中特殊地板测试
// ============================================================

/// 回放编码解码保持一致性（多步复杂路径）
#[test]
fn test_replay_roundtrip_complex() {
    let mut replay = ReplayData::new(42, "hash_xyz".to_string());
    let directions = vec![
        Direction::Up, Direction::Up, Direction::Right,
        Direction::Down, Direction::Left, Direction::Left,
        Direction::Down, Direction::Down, Direction::Right,
        Direction::Right, Direction::Up,
    ];
    for dir in &directions {
        replay.record_move(*dir);
    }

    let encoded = replay.encode();
    let decoded = ReplayData::decode(&encoded).unwrap();

    assert_eq!(decoded.level_id, 42);
    assert_eq!(decoded.total_steps, 11);
    assert_eq!(decoded.moves.len(), 11);
    for (a, b) in decoded.moves.iter().zip(directions.iter()) {
        assert_eq!(a, b);
    }
}
