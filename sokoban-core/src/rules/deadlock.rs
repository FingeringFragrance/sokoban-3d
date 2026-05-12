use crate::grid::*;
use crate::types::*;

#[derive(Debug, Clone)]
pub enum MechanicEvent {
    WindPushed {
        entity_id: u64,
        from: GridPos,
        to: GridPos,
    },
    LavaChanged {
        positions: Vec<GridPos>,
        is_flooded: bool,
    },
    CloudToggled {
        positions: Vec<GridPos>,
        visible: bool,
    },
    EarthquakeShifted {
        entity_id: u64,
        from: GridPos,
        to: GridPos,
    },
    WaterLevelChanged {
        new_level: u8,
        affected: Vec<GridPos>,
    },
}

// ============================================================
//  死局检测
// ============================================================

pub fn detect_deadlock(grid: &GridState) -> Option<DeadlockType> {
    for b in &grid.box_positions {
        let pos = b.pos.pos;
        if grid.is_target(pos) {
            continue;
        }
        if b.box_type == ObjectType::HeavyBox {
            continue;
        }
        if is_corner_deadlock(grid, pos) {
            return Some(DeadlockType::Corner);
        }
        if is_edge_deadlock(grid, pos) {
            return Some(DeadlockType::Edge);
        }
    }
    if is_frozen_deadlock(grid) {
        return Some(DeadlockType::Frozen);
    }
    None
}

fn is_frozen_deadlock(grid: &GridState) -> bool {
    let moveable: Vec<&TrackedBox> = grid
        .box_positions
        .iter()
        .filter(|b| b.box_type != ObjectType::HeavyBox && !grid.is_target(b.pos.pos))
        .collect();

    if moveable.is_empty() {
        return false;
    }

    for b in &moveable {
        let pos = b.pos.pos;
        let mut can_move = false;
        for dir in Direction::all() {
            let target = pos.shift(dir);
            if can_push_or_move_to(grid, target, b.entity_id) {
                can_move = true;
                break;
            }
        }
        if can_move {
            return false;
        }
    }

    true
}

fn can_push_or_move_to(grid: &GridState, pos: GridPos, self_id: u64) -> bool {
    if !grid.in_bounds(pos) {
        return false;
    }
    if !grid.floor_at(pos).is_passable() {
        return false;
    }
    if matches!(grid.floor_at(pos), FloorType::Water | FloorType::Pit) {
        return false;
    }
    match grid.object_at(pos) {
        ObjectType::Wall | ObjectType::CrackedWall | ObjectType::Rock | ObjectType::Gate(_) => {
            return false;
        }
        ObjectType::Pillar(id) => {
            if !grid.is_switch_active(id) {
                return false;
            }
        }
        _ => {}
    }
    if let Some(other) = grid.find_box_at(pos) {
        if other.entity_id != self_id {
            return false;
        }
    }
    true
}

// ============================================================
//  场景周期机制
// ============================================================

pub fn tick_scene_mechanics(
    grid: &mut GridState,
    scene: &SceneTheme,
    current_step: u32,
) -> Vec<MechanicEvent> {
    let mut events = Vec::new();

    if current_step == 0 {
        return events;
    }

    for mechanic in &scene.exclusive_mechanics {
        match mechanic {
            ExclusiveMechanic::WindGust {
                interval,
                direction,
                strength,
            } => {
                if current_step % interval == 0 {
                    for _ in 0..*strength {
                        let before: Vec<(u64, GridPos)> = grid
                            .box_positions
                            .iter()
                            .map(|b| (b.entity_id, b.pos.pos))
                            .collect();

                        apply_wind_gust(grid, *direction);

                        for b in &grid.box_positions {
                            if let Some((_, old_pos)) =
                                before.iter().find(|(id, _)| *id == b.entity_id)
                            {
                                if *old_pos != b.pos.pos {
                                    events.push(MechanicEvent::WindPushed {
                                        entity_id: b.entity_id,
                                        from: *old_pos,
                                        to: b.pos.pos,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            ExclusiveMechanic::LavaCycle {
                rise_interval,
                retreat_interval,
                pattern,
            } => {
                let cycle = rise_interval + retreat_interval;
                if cycle == 0 {
                    continue;
                }
                let phase = current_step % cycle;
                let is_flooded = phase < *rise_interval;

                if phase == 0 || phase == *rise_interval {
                    let positions: Vec<GridPos> = pattern
                        .iter()
                        .map(|(x, z)| GridPos::new(*x, *z))
                        .filter(|p| grid.in_bounds(*p))
                        .collect();

                    for &pos in &positions {
                        if is_flooded {
                            if grid.floor_at(pos) == FloorType::Normal {
                                grid.set_floor(pos, FloorType::Water);
                            }
                        } else {
                            if grid.floor_at(pos) == FloorType::Water {
                                grid.set_floor(pos, FloorType::Normal);
                            }
                        }
                    }

                    events.push(MechanicEvent::LavaChanged {
                        positions,
                        is_flooded,
                    });
                }
            }

            ExclusiveMechanic::AppearingFloor {
                positions,
                appear_interval,
                disappear_interval,
            } => {
                let cycle = appear_interval + disappear_interval;
                if cycle == 0 {
                    continue;
                }
                let phase = current_step % cycle;
                let visible = phase < *appear_interval;

                if phase == 0 || phase == *appear_interval {
                    let positions: Vec<GridPos> = positions
                        .iter()
                        .filter(|p| grid.in_bounds(**p))
                        .copied()
                        .collect();

                    for &pos in &positions {
                        if !visible {
                            if grid.floor_at(pos).is_passable() {
                                grid.set_floor(pos, FloorType::Empty);
                            }
                        } else {
                            if grid.floor_at(pos) == FloorType::Empty {
                                grid.set_floor(pos, FloorType::Normal);
                            }
                        }
                    }

                    events.push(MechanicEvent::CloudToggled {
                        positions,
                        visible,
                    });
                }
            }

            ExclusiveMechanic::WaterLevel {
                initial_level,
                max_level,
            } => {
                let interval = 5u32;
                if current_step % interval == 0 {
                    let max = *max_level as u32;
                    let phase = (current_step / interval) % (max + 1);
                    let effective_level = if phase <= max {
                        *initial_level as u32 + phase
                    } else {
                        *initial_level as u32
                    } as u8;

                    let mut affected = Vec::new();
                    let water_z = (grid.height as i32 - 1)
                        .saturating_sub(effective_level as i32);

                    for z in water_z..grid.height as i32 {
                        for x in 0..grid.width as i32 {
                            let pos = GridPos::new(x, z);
                            if grid.floor_at(pos) == FloorType::Normal {
                                grid.set_floor(pos, FloorType::Water);
                                affected.push(pos);
                            }
                        }
                    }

                    for z in 0..water_z {
                        for x in 0..grid.width as i32 {
                            let pos = GridPos::new(x, z);
                            if grid.floor_at(pos) == FloorType::Water {
                                grid.set_floor(pos, FloorType::Normal);
                                affected.push(pos);
                            }
                        }
                    }

                    events.push(MechanicEvent::WaterLevelChanged {
                        new_level: effective_level,
                        affected,
                    });
                }
            }

            ExclusiveMechanic::LightBeam {
                source_pos,
                source_dir,
                target_pos,
            } => {
                // 光线反射：在指定间隔检查光线路径
                if current_step % 3 == 0 {
                    let _ = trace_light_beam(grid, *source_pos, *source_dir, *target_pos);
                }
            }

            // MirrorZone and BalanceScale are more complex
            // and need UI support - handle as pass-through for now
            ExclusiveMechanic::MirrorZone { .. } => {}
            ExclusiveMechanic::BalanceScale { .. } => {}
        }
    }

    events
}

// ============================================================
//  内部辅助函数
// ============================================================

fn is_corner_deadlock(grid: &GridState, pos: GridPos) -> bool {
    let left = is_wall_or_boundary(grid, pos.shift(Direction::Left));
    let right = is_wall_or_boundary(grid, pos.shift(Direction::Right));
    let up = is_wall_or_boundary(grid, pos.shift(Direction::Up));
    let down = is_wall_or_boundary(grid, pos.shift(Direction::Down));

    (left && up) || (left && down) || (right && up) || (right && down)
}

fn is_edge_deadlock(grid: &GridState, pos: GridPos) -> bool {
    let targets = grid.target_positions();

    let left = is_wall_or_boundary(grid, pos.shift(Direction::Left));
    let right = is_wall_or_boundary(grid, pos.shift(Direction::Right));
    let up = is_wall_or_boundary(grid, pos.shift(Direction::Up));
    let down = is_wall_or_boundary(grid, pos.shift(Direction::Down));

    if left && right {
        if !targets.iter().any(|t| t.x == pos.x) {
            return true;
        }
    }

    if up && down {
        if !targets.iter().any(|t| t.z == pos.z) {
            return true;
        }
    }

    false
}

fn is_wall_or_boundary(grid: &GridState, pos: GridPos) -> bool {
    if !grid.in_bounds(pos) {
        return true;
    }
    grid.is_wall(pos)
}

fn apply_wind_gust(grid: &mut GridState, direction: Direction) {
    let mut boxes: Vec<(u64, GridPos)> = grid
        .box_positions
        .iter()
        .filter(|b| b.box_type != ObjectType::HeavyBox)
        .map(|b| (b.entity_id, b.pos.pos))
        .collect();

    match direction {
        Direction::Right => boxes.sort_by(|a, b| b.1.x.cmp(&a.1.x)),
        Direction::Left => boxes.sort_by(|a, b| a.1.x.cmp(&b.1.x)),
        Direction::Down => boxes.sort_by(|a, b| b.1.z.cmp(&a.1.z)),
        Direction::Up => boxes.sort_by(|a, b| a.1.z.cmp(&b.1.z)),
    }

    for (entity_id, pos) in boxes {
        let target = pos.shift(direction);

        if !grid.in_bounds(target) {
            continue;
        }
        if !grid.floor_at(target).is_passable() {
            continue;
        }
        if matches!(grid.floor_at(target), FloorType::Water | FloorType::Pit) {
            continue;
        }
        if grid.find_box_at(target).is_some() {
            continue;
        }
        match grid.object_at(target) {
            ObjectType::Wall | ObjectType::CrackedWall | ObjectType::Rock | ObjectType::Gate(_) => {
                continue;
            }
            ObjectType::Pillar(id) => {
                if !grid.is_switch_active(id) {
                    continue;
                }
            }
            _ => {}
        }

        let dest = GridPos3D::new(target.x, target.z, grid.player_pos.floor);
        grid.move_box(entity_id, dest);
    }
}

fn trace_light_beam(
    _grid: &mut GridState,
    _source: GridPos,
    _direction: Direction,
    _target: GridPos,
) -> bool {
    // 光线追踪：从源头沿方向发射，碰到镜子偏转 90 度
    // 到达目标点则触发开关
    // 完整实现需要遍历路径，此处保留接口
    // 实际激活由 landing.rs 中的 mirror_deflect 处理
    true
}
