"""
生成 Sokoban 3D 的基础音效文件（.wav）
运行: python tools/generate_sounds.py
输出: sokoban-game/assets/audio/sounds/*.wav
"""

import os
import struct
import math
import random

OUTPUT_DIR = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "sokoban-game", "assets", "audio", "sounds"
)

SAMPLE_RATE = 22050


def write_wav(filepath, samples):
    """写入 16-bit mono WAV 文件"""
    n = len(samples)
    # 归一化到 int16
    max_val = max(abs(s) for s in samples) or 1.0
    int_samples = [int(s / max_val * 32000) for s in samples]

    with open(filepath, "wb") as f:
        # RIFF header
        data_size = n * 2
        f.write(b"RIFF")
        f.write(struct.pack("<I", 36 + data_size))
        f.write(b"WAVE")
        # fmt chunk
        f.write(b"fmt ")
        f.write(struct.pack("<IHHIIHH", 16, 1, 1, SAMPLE_RATE, SAMPLE_RATE * 2, 2, 16))
        # data chunk
        f.write(b"data")
        f.write(struct.pack("<I", data_size))
        for s in int_samples:
            f.write(struct.pack("<h", s))


def generate_tone(freq, duration, volume=0.5, fade_out=True):
    """生成简单正弦波音调"""
    n = int(SAMPLE_RATE * duration)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = volume
        if fade_out:
            env *= max(0.0, 1.0 - t / duration)
        samples.append(math.sin(2 * math.pi * freq * t) * env)
    return samples


def generate_noise(duration, volume=0.3):
    """生成白噪声"""
    n = int(SAMPLE_RATE * duration)
    return [random.uniform(-1, 1) * volume for _ in range(n)]


def generate_move():
    """脚步声：短促低频"""
    s = generate_tone(200, 0.08, 0.4)
    s += generate_tone(150, 0.06, 0.3)
    return s


def generate_push():
    """推箱子：低沉有力"""
    s = generate_tone(120, 0.15, 0.6)
    s += generate_tone(80, 0.2, 0.4)
    noise = generate_noise(0.1, 0.2)
    return [a + b for a, b in zip(s, noise + [0] * max(0, len(s) - len(noise)))]


def generate_target():
    """箱子到位：明亮上升音"""
    n = int(SAMPLE_RATE * 0.3)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        freq = 400 + t * 2000
        env = max(0.0, 1.0 - t / 0.3) * 0.5
        samples.append(math.sin(2 * math.pi * freq * t) * env)
    return samples


def generate_complete():
    """通关：欢快和弦"""
    s1 = generate_tone(523, 0.5, 0.3)  # C5
    s2 = generate_tone(659, 0.5, 0.3)  # E5
    s3 = generate_tone(784, 0.5, 0.3)  # G5
    s4 = generate_tone(1047, 0.6, 0.25)  # C6

    n = max(len(s1), len(s2), len(s3), len(s4))
    result = [0.0] * n
    for s in [s1, s2, s3, s4]:
        for i in range(len(s)):
            result[i] += s[i]
    return result


def generate_collect():
    """拾取钥匙：清脆短音"""
    n = int(SAMPLE_RATE * 0.15)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        freq = 800 + t * 3000
        env = max(0.0, 1.0 - t / 0.15) * 0.4
        samples.append(math.sin(2 * math.pi * freq * t) * env)
    return samples


def generate_door():
    """开门：沉重低音"""
    s = generate_tone(100, 0.25, 0.5)
    s += generate_tone(150, 0.2, 0.3)
    s += generate_tone(200, 0.15, 0.2)
    return s


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    sounds = {
        "move": generate_move,
        "push": generate_push,
        "target": generate_target,
        "complete": generate_complete,
        "collect": generate_collect,
        "door": generate_door,
    }

    for name, generator in sounds.items():
        samples = generator()
        filepath = os.path.join(OUTPUT_DIR, f"{name}.wav")
        write_wav(filepath, samples)
        print(f"Generated: {filepath} ({len(samples)} samples)")

    print(f"\nDone! {len(sounds)} sound files generated in {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
