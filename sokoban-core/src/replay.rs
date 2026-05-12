use serde::{Deserialize, Serialize};

use crate::types::Direction;

/// 回放数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayData {
    pub level_id: u32,
    pub level_hash: String,
    pub moves: Vec<Direction>,
    pub total_steps: u32,
    pub total_time_ms: u64,
    pub player_name: String,
    pub timestamp: u64,
}

impl ReplayData {
    pub fn new(level_id: u32, level_hash: String) -> Self {
        Self {
            level_id,
            level_hash,
            moves: Vec::new(),
            total_steps: 0,
            total_time_ms: 0,
            player_name: String::new(),
            timestamp: 0,
        }
    }

    pub fn record_move(&mut self, dir: Direction) {
        self.moves.push(dir);
        self.total_steps += 1;
    }

    pub fn pop_move(&mut self) {
        if self.moves.pop().is_some() {
            self.total_steps = self.total_steps.saturating_sub(1);
        }
    }

    /// 编码为紧凑字符串（方便分享）
    pub fn encode(&self) -> String {
        let mut bits = Vec::new();
        for dir in &self.moves {
            let val: u8 = match dir {
                Direction::Up => 0b00,
                Direction::Down => 0b01,
                Direction::Left => 0b10,
                Direction::Right => 0b11,
            };
            bits.push(val);
        }

        let mut bytes = Vec::new();
        for chunk in bits.chunks(4) {
            let mut byte: u8 = 0;
            for (i, &b) in chunk.iter().enumerate() {
                byte |= b << (i * 2);
            }
            bytes.push(byte);
        }

        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let checksum = bytes.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));

        format!("R1:{}:{}:{}:{:02x}", self.level_id, self.total_steps, hex, checksum)
    }

    /// 从字符串解码
    pub fn decode(encoded: &str) -> Option<Self> {
        let parts: Vec<&str> = encoded.split(':').collect();
        if parts.len() < 4 || parts[0] != "R1" {
            return None;
        }

        let level_id: u32 = parts[1].parse().ok()?;
        let total_steps: u32 = parts[2].parse().ok()?;
        let hex = parts[3];

        let mut bytes = Vec::new();
        for i in (0..hex.len()).step_by(2) {
            if i + 2 > hex.len() {
                break;
            }
            let byte = u8::from_str_radix(&hex[i..i + 2], 16).ok()?;
            bytes.push(byte);
        }

        if parts.len() >= 5 {
            let expected_checksum = u8::from_str_radix(parts[4], 16).ok()?;
            let actual_checksum = bytes.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
            if expected_checksum != actual_checksum {
                return None;
            }
        }

        let mut moves = Vec::new();
        for byte in &bytes {
            for i in 0..4 {
                if moves.len() >= total_steps as usize {
                    break;
                }
                let val = (byte >> (i * 2)) & 0b11;
                let dir = match val {
                    0b00 => Direction::Up,
                    0b01 => Direction::Down,
                    0b10 => Direction::Left,
                    0b11 => Direction::Right,
                    _ => return None,
                };
                moves.push(dir);
            }
        }

        Some(Self {
            level_id,
            level_hash: String::new(),
            moves,
            total_steps,
            total_time_ms: 0,
            player_name: String::new(),
            timestamp: 0,
        })
    }

    /// 验证回放步数是否匹配
    pub fn is_valid(&self) -> bool {
        self.moves.len() == self.total_steps as usize
    }
}

/// 回放播放器状态
#[derive(Debug, Clone)]
pub struct ReplayPlayer {
    pub replay: ReplayData,
    pub current_step: usize,
    pub is_playing: bool,
    pub speed: f32,
}

impl ReplayPlayer {
    pub fn new(replay: ReplayData) -> Self {
        Self {
            replay,
            current_step: 0,
            is_playing: false,
            speed: 1.0,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.is_playing = false;
    }

    /// 获取下一步方向
    pub fn next_move(&mut self) -> Option<Direction> {
        if !self.is_playing {
            return None;
        }
        if self.current_step >= self.replay.moves.len() {
            self.is_playing = false;
            return None;
        }
        let dir = self.replay.moves[self.current_step];
        self.current_step += 1;
        if self.current_step >= self.replay.moves.len() {
            self.is_playing = false;
        }
        Some(dir)
    }

    pub fn is_finished(&self) -> bool {
        self.current_step >= self.replay.moves.len()
    }

    pub fn progress(&self) -> f32 {
        if self.replay.moves.is_empty() {
            return 1.0;
        }
        self.current_step as f32 / self.replay.moves.len() as f32
    }
}
