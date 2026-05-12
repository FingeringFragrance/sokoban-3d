/// 缓动与插值工具函数

/// 指数衰减插值（平滑跟随）
/// speed 越大收敛越快
pub fn exp_decay(current: f32, target: f32, speed: f32, dt: f32) -> f32 {
    let t = 1.0 - (-speed * dt).exp();
    current + (target - current) * t
}

/// 计算抖动偏移量
/// 返回 (x_offset, z_offset)
pub fn shake_offset(timer: f32, duration: f32, intensity: f32, time_secs: f32) -> (f32, f32) {
    if timer <= 0.0 {
        return (0.0, 0.0);
    }
    let progress = (timer / duration).clamp(0.0, 1.0);
    let amp = intensity * progress * progress; // 二次衰减
    let phase = time_secs * 45.0; // 快速振荡
    (phase.sin() * amp, phase.cos() * amp * 0.7)
}

/// ease-out-back: 先超过目标再回弹
#[allow(dead_code)]
pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158f32;
    let c3 = c1 + 1.0;
    let t1 = t - 1.0;
    1.0 + c3 * t1 * t1 * t1 + c1 * t1 * t1
}

/// ease-out-cubic: 平滑减速
#[allow(dead_code)]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = 1.0 - t.clamp(0.0, 1.0);
    1.0 - t * t * t
}
