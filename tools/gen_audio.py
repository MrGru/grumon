#!/usr/bin/env python3
"""Generates the original music and sound effects of THIÊN MỆNH: TÀN HỒN.

Everything is synthesised from scratch (numpy) with fixed seeds, so the output
is reproducible and owned by the project (see docs/asset-manifest.md):

  assets/audio/music/*.ogg   seamless loops (đàn tranh, sáo trúc, trống, cồng)
  assets/audio/sfx/*.ogg     interface and battle sounds, short jingles

Requires numpy and ffmpeg (with libvorbis). Run from the repository root:
    python3 tools/gen_audio.py
"""

import math
import random
import shutil
import subprocess
import tempfile
import wave
from pathlib import Path

import numpy as np

SR = 32000
ROOT = Path("assets/audio")

# ---------------------------------------------------------------------------
# Instruments
# ---------------------------------------------------------------------------


def t_axis(dur):
    return np.arange(int(dur * SR)) / SR


def lowpass(x, k):
    """Cheap low-pass: moving average of width k samples."""
    if k <= 1:
        return x
    kernel = np.ones(k) / k
    return np.convolve(x, kernel, mode="same")


def env_adsr(n, a, d, s, r, sustain_len=None):
    a, d, r = int(a * SR), int(d * SR), int(r * SR)
    if sustain_len is None:
        sustain_len = max(0, n - a - d - r)
    e = np.concatenate([
        np.linspace(0, 1, max(a, 1), endpoint=False),
        np.linspace(1, s, max(d, 1), endpoint=False),
        np.full(max(sustain_len, 0), s),
        np.linspace(s, 0, max(r, 1)),
    ])
    if len(e) < n:
        e = np.concatenate([e, np.zeros(n - len(e))])
    return e[:n]


def tranh(freq, dur, bend=0.0, vibrato=True, bright=1.0):
    """Đàn tranh pluck: decaying harmonics, optional opening bend (nhấn)."""
    ring = max(dur, 0.6) + 0.8
    t = t_axis(ring)
    f = np.full_like(t, freq)
    if bend:
        f *= 2 ** (bend * np.exp(-t / 0.05) / 12)
    if vibrato and dur > 0.35:
        depth = np.clip((t - 0.2) / 0.3, 0, 1) * 0.004
        f *= 1 + depth * np.sin(2 * math.pi * 5.5 * t)
    phase = 2 * math.pi * np.cumsum(f) / SR
    out = np.zeros_like(t)
    for n in range(1, 11):
        amp = abs(math.sin(math.pi * n * 0.18)) / n ** (1.25 - 0.2 * bright)
        decay = 1.1 + 0.9 * (n - 1)
        out += amp * np.sin(n * phase * (1 + 0.0008 * (n - 1))) * np.exp(-decay * t)
    attack = np.clip(t / 0.003, 0, 1)
    # Damp the string a little after the written duration.
    damp = np.where(t > dur + 0.25, np.exp(-(t - dur - 0.25) * 6), 1.0)
    return out * attack * damp * 0.32


def sao(freq, dur, rng, vib=0.006):
    """Sáo trúc (bamboo flute): breathy sine with delayed vibrato."""
    t = t_axis(dur + 0.15)
    depth = np.clip((t - 0.22) / 0.3, 0, 1) * vib
    f = freq * (1 + depth * np.sin(2 * math.pi * 5.2 * t))
    # Small scoop into the note.
    f *= 2 ** (-0.35 * np.exp(-t / 0.04) / 12)
    phase = 2 * math.pi * np.cumsum(f) / SR
    tone = np.sin(phase) + 0.32 * np.sin(2 * phase) + 0.1 * np.sin(3 * phase) + 0.04 * np.sin(4 * phase)
    noise = lowpass(rng.standard_normal(len(t)), 6) * 0.10
    env = env_adsr(len(t), 0.07, 0.1, 0.82, 0.14)
    return (tone + noise) * env * 0.26


def drone(freq, dur, rng):
    t = t_axis(dur)
    lfo = 0.75 + 0.25 * np.sin(2 * math.pi * 0.11 * t + rng.random() * 6)
    tone = (np.sin(2 * math.pi * freq * t) + 0.5 * np.sin(2 * math.pi * freq * 1.5 * t)
            + 0.25 * np.sin(2 * math.pi * freq * 2 * t))
    return tone * lfo * env_adsr(len(t), 1.5, 0.1, 1.0, 1.5) * 0.09


def trong(kind, rng):
    """Drums: 'big' (trống cái), 'small' (trống con), 'mo' (wood block), 'cymbal'."""
    if kind == "big":
        t = t_axis(0.7)
        f = 44 + 36 * np.exp(-t / 0.05)
        body = np.sin(2 * math.pi * np.cumsum(f) / SR) * np.exp(-t / 0.22)
        click = rng.standard_normal(len(t)) * np.exp(-t / 0.008) * 0.3
        return (body + lowpass(click, 4)) * 0.75
    if kind == "small":
        t = t_axis(0.3)
        f = 150 + 80 * np.exp(-t / 0.03)
        body = np.sin(2 * math.pi * np.cumsum(f) / SR) * np.exp(-t / 0.08)
        snap = lowpass(rng.standard_normal(len(t)), 2) * np.exp(-t / 0.03) * 0.35
        return (body + snap) * 0.45
    if kind == "mo":
        t = t_axis(0.12)
        tone = np.sin(2 * math.pi * 820 * t) + 0.5 * np.sin(2 * math.pi * 1310 * t)
        return tone * np.exp(-t / 0.018) * 0.28
    if kind == "cymbal":
        t = t_axis(0.9)
        n = rng.standard_normal(len(t))
        hp = n - lowpass(n, 3)
        return hp * np.exp(-t / 0.22) * 0.18
    raise ValueError(kind)


def cong(freq, rng, dur=4.5):
    """Gong (cồng chiêng): inharmonic partials, long decay."""
    t = t_axis(dur)
    out = np.zeros_like(t)
    for ratio, amp, dec in [(1, 1, 2.6), (1.48, 0.6, 1.9), (2.03, 0.45, 1.5), (2.6, 0.3, 1.1),
                            (3.27, 0.22, 0.8), (4.1, 0.12, 0.5)]:
        drift = 1 + 0.002 * np.exp(-t / 0.4)
        out += amp * np.sin(2 * math.pi * freq * ratio * drift * t + rng.random() * 6) * np.exp(-t / dec)
    attack = np.clip(t / 0.012, 0, 1)
    return out * attack * 0.28


# ---------------------------------------------------------------------------
# Mixing helpers
# ---------------------------------------------------------------------------


def reverb(x, rng, rt=1.8, wet=0.22):
    n = int(rt * SR)
    t = np.arange(n) / SR
    ir = rng.standard_normal(n) * np.exp(-6.9 * t / rt)
    ir = lowpass(ir, 5)
    ir[: int(0.012 * SR)] = 0  # pre-delay
    ir /= np.sqrt(np.sum(ir ** 2))
    size = 1 << int(math.ceil(math.log2(len(x) + n)))
    y = np.fft.irfft(np.fft.rfft(x, size) * np.fft.rfft(ir, size), size)[: len(x)]
    return x * (1 - wet) + y * wet * 0.9


def soft_clip(x):
    return np.tanh(x)


def normalize(x, peak):
    m = np.max(np.abs(x))
    return x if m == 0 else x * (peak / m)


class Track:
    """A loop of `bars` bars; notes past the end wrap to the start."""

    def __init__(self, bpm, bars, beats_per_bar=4, tail=4.0):
        self.beat = 60.0 / bpm
        self.length = int(bars * beats_per_bar * self.beat * SR)
        self.buf = np.zeros(self.length + int(tail * SR))

    def at(self, beat):
        return int(beat * self.beat * SR)

    def add(self, wave_, beat, gain=1.0):
        start = self.at(beat)
        end = min(start + len(wave_), len(self.buf))
        self.buf[start:end] += wave_[: end - start] * gain

    def render(self, rng, rt=1.8, wet=0.22, peak=0.62):
        y = reverb(self.buf, rng, rt, wet)
        loop = y[: self.length].copy()
        tail = y[self.length:]
        loop[: len(tail)] += tail[: self.length]
        return normalize(soft_clip(normalize(loop, 1.1)), peak)


class Scale:
    def __init__(self, root_hz, steps):
        self.root = root_hz
        self.steps = steps

    def hz(self, degree):
        octave, idx = divmod(degree, len(self.steps))
        return self.root * 2 ** ((12 * octave + self.steps[idx]) / 12)


PENTA_MAJOR = [0, 2, 4, 7, 9]
PENTA_MINOR = [0, 3, 5, 7, 10]
# Oán-like mode (Vietnamese "hơi oán" colour): flattened 2nd and 6th.
OAN = [0, 1, 5, 7, 8]


def phrase(rng, length_beats, start, low, high, rhythm, end_on=None):
    """Random-walk melody: list of (beat, dur, degree)."""
    notes, beat, deg = [], 0.0, start
    while beat < length_beats - 1e-6:
        dur = rng.choice(rhythm)
        dur = min(dur, length_beats - beat)
        step = rng.choices([-2, -1, 0, 1, 2, 3], weights=[2, 5, 1, 5, 2, 1])[0]
        deg = max(low, min(high, deg + step))
        notes.append((beat, dur, deg))
        beat += dur
    if end_on is not None and notes:
        b, d, _ = notes[-1]
        notes[-1] = (b, d, end_on)
    return notes


def play_melody(track, notes, scale, offset, voice, rng, gain=1.0, **kw):
    for beat, dur, deg in notes:
        hz = scale.hz(deg)
        secs = dur * track.beat
        if voice == "tranh":
            bend = 1.0 if rng.random() < 0.18 and secs > 0.3 else 0.0
            track.add(tranh(hz, secs, bend=bend, **kw), offset + beat, gain)
        else:
            track.add(sao(hz, secs * 0.95, np.random.default_rng(int(hz * 100) + int(beat * 7))), offset + beat, gain)


def form(rng, beats, start, low, high, rhythm, root):
    """A A' B A'' melody over 4 sections of `beats` beats."""
    a = phrase(rng, beats, start, low, high, rhythm)
    a2 = a[:-1] + [(a[-1][0], a[-1][1], root + 2)]
    b = phrase(rng, beats, start + 2, low + 1, high, rhythm)
    a3 = a[:-1] + [(a[-1][0], a[-1][1], root)]
    return [(a, 0), (a2, beats), (b, 2 * beats), (a3, 3 * beats)]


# ---------------------------------------------------------------------------
# Music
# ---------------------------------------------------------------------------


def arpeggio(track, scale, bars, degrees, rng, gain=0.5, step=1.0, beats_per_bar=4):
    beat = 0.0
    total = bars * beats_per_bar
    i = 0
    while beat < total:
        track.add(tranh(scale.hz(degrees[i % len(degrees)]), step * track.beat * 1.6, vibrato=False, bright=0.6), beat, gain)
        beat += step
        i += 1


def music_title(rng):
    s = Scale(146.83, PENTA_MINOR)  # D
    tr = Track(bpm=66, bars=16)
    for bar in range(16):
        tr.add(drone(s.hz(0) / 2, 4 * tr.beat + 1.0, rng), bar * 4, 0.9)
    arpeggio(tr, s, 16, [0, 2, 4, 5, 7, 5, 4, 2], rng, gain=0.42, step=0.5)
    for notes, off in form(rng, 16, 7, 5, 11, [1, 1, 1.5, 0.5, 2, 3], 5):
        play_melody(tr, notes, s, off, "sao", rng, gain=0.9)
    for bar in (0, 8):
        tr.add(cong(s.hz(0) / 2, rng), bar * 4, 0.5)
    return tr.render(rng, rt=2.6, wet=0.32)


def music_village(rng):
    s = Scale(196.0, PENTA_MAJOR)  # G
    tr = Track(bpm=100, bars=16)
    pattern = [0, 4, 2, 4, 1, 4, 2, 4]
    arpeggio(tr, s, 16, pattern, rng, gain=0.30, step=0.5)
    for notes, off in form(rng, 16, 7, 5, 12, [0.5, 0.5, 1, 1, 1.5, 0.5], 5):
        play_melody(tr, notes, s, off, "tranh", rng, gain=0.85)
    for beat in range(64):
        if beat % 2 == 1:
            tr.add(trong("mo", rng), beat, 0.7)
        if beat % 8 == 0:
            tr.add(trong("small", rng), beat, 0.6)
    flute = phrase(random.Random(9), 16, 9, 7, 12, [2, 2, 4])
    play_melody(tr, flute, s, 32, "sao", rng, gain=0.45)
    return tr.render(rng, rt=1.4, wet=0.18)


def music_night(rng):
    s = Scale(164.81, PENTA_MINOR)  # E
    tr = Track(bpm=58, bars=12)
    for bar in range(0, 12, 2):
        tr.add(drone(s.hz(0) / 2, 8 * tr.beat + 1.5, rng), bar * 4, 1.0)
    sparse = random.Random(4)
    for bar in range(12):
        for k in range(2):
            if sparse.random() < 0.7:
                deg = sparse.choice([0, 2, 3, 4, 5, 7])
                tr.add(tranh(s.hz(deg), 1.6, bend=1.0 if sparse.random() < 0.3 else 0.0), bar * 4 + k * 2 + sparse.choice([0, 0.5]), 0.6)
    for notes, off in form(rng, 12, 7, 5, 10, [2, 1, 3, 1.5, 0.5], 5):
        play_melody(tr, notes, s, off, "sao", rng, gain=0.6)
    return tr.render(rng, rt=3.0, wet=0.38, peak=0.5)


def music_forest(rng):
    s = Scale(220.0, PENTA_MINOR)  # A
    tr = Track(bpm=84, bars=16)
    arpeggio(tr, s, 16, [0, 2, 3, 4, 3, 2], rng, gain=0.28, step=0.5)
    for bar in range(0, 16, 4):
        tr.add(drone(s.hz(0) / 2, 16 * tr.beat + 1.0, rng), bar * 4, 0.7)
    for notes, off in form(rng, 16, 5, 3, 10, [1, 1, 0.5, 0.5, 2, 1.5], 5):
        play_melody(tr, notes, s, off, "sao", rng, gain=0.8)
    for beat in range(0, 64, 4):
        tr.add(trong("small", rng), beat, 0.25)
    return tr.render(rng, rt=2.2, wet=0.28)


def battle_drums(tr, rng, bars, heavy=False):
    for bar in range(bars):
        b = bar * 4
        for x in (0, 1.5, 2.5) if not heavy else (0, 0.75, 1.5, 2, 2.5, 3.5):
            tr.add(trong("big", rng), b + x, 0.8)
        for x in (1, 3):
            tr.add(trong("small", rng), b + x, 0.75)
        for x in np.arange(0, 4, 0.5):
            tr.add(trong("mo", rng), b + x, 0.25)
        if bar % 4 == 0:
            tr.add(trong("cymbal", rng), b, 0.7)


def music_raid(rng):
    s = Scale(146.83, PENTA_MINOR)  # D
    tr = Track(bpm=128, bars=16)
    battle_drums(tr, rng, 16, heavy=True)
    arpeggio(tr, s, 16, [0, 0, 3, 0, 2, 0, 4, 3], rng, gain=0.4, step=0.5)
    for bar in range(0, 16, 4):
        tr.add(cong(s.hz(0) / 2, rng, 3.0), bar * 4, 0.6)
    for notes, off in form(rng, 16, 7, 5, 11, [0.5, 0.5, 1, 1.5, 0.5, 2], 5):
        play_melody(tr, notes, s, off, "sao", rng, gain=0.55)
    return tr.render(rng, rt=1.6, wet=0.2)


def music_battle(rng):
    s = Scale(164.81, PENTA_MINOR)  # E
    tr = Track(bpm=144, bars=16)
    battle_drums(tr, rng, 16)
    arpeggio(tr, s, 16, [0, 2, 3, 2, 4, 3, 2, 1], rng, gain=0.38, step=0.5)
    for notes, off in form(rng, 16, 7, 5, 12, [0.5, 0.5, 1, 1, 0.5, 1.5], 5):
        play_melody(tr, notes, s, off, "tranh", rng, gain=0.8, bright=1.3)
    return tr.render(rng, rt=1.2, wet=0.16)


def music_boss(rng):
    s = Scale(130.81, OAN)  # C, hơi oán
    tr = Track(bpm=116, bars=16)
    battle_drums(tr, rng, 16, heavy=True)
    for bar in range(16):
        tr.add(drone(s.hz(0) / 2, 4 * tr.beat + 0.5, rng), bar * 4, 1.0)
    arpeggio(tr, s, 16, [0, 1, 0, 3, 0, 1, 4, 3], rng, gain=0.42, step=0.5)
    for bar in range(0, 16, 2):
        tr.add(cong(s.hz(0) / 2, rng, 3.5), bar * 4, 0.55)
    for notes, off in form(rng, 16, 7, 5, 11, [1, 0.5, 0.5, 1.5, 2, 0.5], 5):
        play_melody(tr, notes, s, off, "sao", rng, gain=0.7)
    return tr.render(rng, rt=1.8, wet=0.22)


def music_sorrow(rng):
    s = Scale(146.83, OAN)  # D, hơi oán
    tr = Track(bpm=54, bars=10)
    for bar in range(0, 10, 2):
        tr.add(drone(s.hz(0) / 2, 8 * tr.beat + 1.5, rng), bar * 4, 0.9)
    for notes, off in form(rng, 10, 5, 3, 9, [2, 1, 1, 3, 1.5, 0.5], 5):
        play_melody(tr, notes, s, off, "tranh", rng, gain=0.8)
    flute = phrase(random.Random(21), 20, 7, 5, 10, [2, 3, 1])
    play_melody(tr, flute, s, 20, "sao", rng, gain=0.5)
    return tr.render(rng, rt=3.2, wet=0.4, peak=0.5)


MUSIC = {
    "title": music_title,
    "village": music_village,
    "night": music_night,
    "forest": music_forest,
    "raid": music_raid,
    "battle": music_battle,
    "boss": music_boss,
    "sorrow": music_sorrow,
}

# ---------------------------------------------------------------------------
# Sound effects
# ---------------------------------------------------------------------------


def blip(freq, dur, decay):
    t = t_axis(dur)
    return np.sin(2 * math.pi * freq * t) * np.exp(-t / decay)


def sweep(f0, f1, dur, rng, noise=0.0):
    t = t_axis(dur)
    f = f0 * (f1 / f0) ** (t / dur)
    tone = np.sin(2 * math.pi * np.cumsum(f) / SR)
    if noise:
        tone += lowpass(rng.standard_normal(len(t)), 3) * noise
    return tone * env_adsr(len(t), 0.01, 0.05, 0.7, dur * 0.5)


def cat(*parts, gap=0.0):
    out = []
    for p in parts:
        out.append(p)
        if gap:
            out.append(np.zeros(int(gap * SR)))
    return np.concatenate(out)


def mix(*parts):
    n = max(len(p) for p in parts)
    out = np.zeros(n)
    for p in parts:
        out[: len(p)] += p
    return out


def sfx_set(rng):
    s = Scale(392.0, PENTA_MAJOR)
    return {
        "ui_move": blip(1320, 0.05, 0.012) * 0.5,
        "ui_confirm": cat(blip(988, 0.05, 0.02), blip(1480, 0.09, 0.03)),
        "ui_cancel": cat(blip(880, 0.05, 0.02), blip(587, 0.09, 0.03)),
        "text": blip(1760, 0.025, 0.006) * 0.35,
        "hit": mix(trong("small", rng), lowpass(rng.standard_normal(int(0.08 * SR)), 2) * np.exp(-t_axis(0.08) / 0.02) * 0.6),
        "hit_heavy": mix(trong("big", rng) * 1.2, trong("cymbal", rng) * 0.8, trong("small", rng)),
        "heal": mix(*[np.pad(tranh(s.hz(d), 0.5, vibrato=False), (int(i * 0.07 * SR), 0)) for i, d in enumerate([5, 7, 9, 12])]),
        "shield": mix(cong(784, rng, 1.2) * 0.8, blip(1568, 0.6, 0.2) * 0.3),
        "charge": sweep(220, 880, 0.45, rng, noise=0.15) * 0.7,
        "interrupt": mix(sweep(1400, 300, 0.3, rng, noise=0.4), trong("small", rng)),
        "formation": cong(196, rng, 2.5),
        "pickup": mix(*[np.pad(tranh(s.hz(d), 0.3, vibrato=False), (int(i * 0.06 * SR), 0)) for i, d in enumerate([7, 9])]),
        "quest": mix(*[np.pad(tranh(s.hz(d), 0.6, vibrato=False), (int(i * 0.1 * SR), 0)) for i, d in enumerate([5, 7, 9, 10])]),
        "save": mix(cong(523, rng, 1.0) * 0.6, blip(1046, 0.4, 0.1) * 0.3),
        "warp": sweep(300, 1200, 0.5, rng, noise=0.6) * 0.5,
        "victory": mix(
            *[np.pad(tranh(s.hz(d), 0.5, vibrato=False, bright=1.3), (int(i * 0.12 * SR), 0)) for i, d in enumerate([5, 7, 9, 10, 12])],
            np.pad(cong(196, rng, 2.0) * 0.6, (int(0.6 * SR), 0)),
        ),
        "defeat": mix(
            *[np.pad(tranh(Scale(196.0, PENTA_MINOR).hz(d), 0.8), (int(i * 0.3 * SR), 0)) for i, d in enumerate([4, 3, 1, 0])],
            np.pad(cong(98, rng, 3.0) * 0.6, (int(0.9 * SR), 0)),
        ),
    }


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------


def write_ogg(samples, path, quality=4):
    path.parent.mkdir(parents=True, exist_ok=True)
    pcm = (np.clip(samples, -1, 1) * 32767).astype(np.int16)
    with tempfile.TemporaryDirectory() as tmp:
        wav = Path(tmp) / "out.wav"
        with wave.open(str(wav), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes(pcm.tobytes())
        subprocess.run(
            ["ffmpeg", "-loglevel", "error", "-y", "-i", str(wav), "-c:a", "libvorbis",
             "-q:a", str(quality), str(path)],
            check=True,
        )


def report(name, x):
    rms = 20 * math.log10(max(float(np.sqrt(np.mean(x ** 2))), 1e-9))
    print(f"{name:14s} {len(x) / SR:6.1f}s  peak {np.max(np.abs(x)):.2f}  rms {rms:6.1f} dBFS")


def main():
    if shutil.which("ffmpeg") is None:
        raise SystemExit("ffmpeg is required")
    for i, (name, fn) in enumerate(MUSIC.items()):
        random.seed(100 + i)
        rng = np.random.default_rng(100 + i)
        # `phrase` uses the `random` module through this Random instance.
        samples = fn(_Rng(100 + i, rng))
        report(name, samples)
        write_ogg(samples, ROOT / "music" / f"{name}.ogg", quality=3)
    rng = np.random.default_rng(7)
    for name, samples in sfx_set(_Rng(7, rng)).items():
        samples = normalize(samples, 0.8)
        fade = min(len(samples), int(0.01 * SR))
        samples[-fade:] *= np.linspace(1, 0, fade)
        report(name, samples)
        write_ogg(samples, ROOT / "sfx" / f"{name}.ogg", quality=4)


class _Rng:
    """numpy Generator plus the `random.Random` methods `phrase` needs."""

    def __init__(self, seed, np_rng):
        self._py = random.Random(seed)
        self._np = np_rng

    def choice(self, seq):
        return self._py.choice(seq)

    def choices(self, seq, weights):
        return self._py.choices(seq, weights=weights)

    def random(self):
        return self._np.random()

    def standard_normal(self, n):
        return self._np.standard_normal(n)


if __name__ == "__main__":
    main()
