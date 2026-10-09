//! Built-in Telex input method, so players can type Vietnamese names even
//! without an OS input method (vietnamese-style-guide §3.4 tone placement).
//!
//! `aa → â, aw → ă, ee → ê, oo → ô, ow → ơ, uw / w → ư, dd → đ`,
//! tones `s` sắc, `f` huyền, `r` hỏi, `x` ngã, `j` nặng, `z` removes the tone.
//! Repeating a key undoes it (`ass → as`, `aaa → aa`).

/// Base vowels (with their own modifier) and their six tone forms:
/// none, sắc, huyền, hỏi, ngã, nặng.
const VOWELS: [[char; 6]; 12] = [
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
];

/// A decomposed letter: lowercase base (with modifier), tone index, case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Letter {
    base: char,
    tone: usize,
    upper: bool,
}

impl Letter {
    fn from_char(c: char) -> Self {
        let upper = c.is_uppercase();
        let lower = c.to_lowercase().next().unwrap_or(c);
        for row in VOWELS {
            if let Some(tone) = row.iter().position(|&v| v == lower) {
                return Letter {
                    base: row[0],
                    tone,
                    upper,
                };
            }
        }
        Letter {
            base: lower,
            tone: 0,
            upper,
        }
    }

    fn to_char(self) -> char {
        let lower = VOWELS
            .iter()
            .find(|row| row[0] == self.base)
            .map_or(self.base, |row| row[self.tone]);
        if self.upper {
            lower.to_uppercase().next().unwrap_or(lower)
        } else {
            lower
        }
    }

    fn is_vowel(self) -> bool {
        VOWELS.iter().any(|row| row[0] == self.base)
    }

    /// Vowel carrying a diacritic other than a tone (ă â ê ô ơ ư).
    fn is_modified(self) -> bool {
        matches!(self.base, 'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư')
    }

    /// The plain Latin letter (â → a, ư → u, đ → d).
    fn plain(self) -> char {
        match self.base {
            'ă' | 'â' => 'a',
            'ê' => 'e',
            'ô' | 'ơ' => 'o',
            'ư' => 'u',
            'đ' => 'd',
            c => c,
        }
    }
}

fn tone_of_key(key: char) -> Option<usize> {
    match key {
        's' => Some(1),
        'f' => Some(2),
        'r' => Some(3),
        'x' => Some(4),
        'j' => Some(5),
        _ => None,
    }
}

/// Indices of the syllable's vowel cluster, treating the `u` of `qu` and the
/// `i` of `gi` (when another vowel follows) as consonants.
fn vowel_cluster(word: &[Letter]) -> Vec<usize> {
    let mut cluster: Vec<usize> = Vec::new();
    for (i, letter) in word.iter().enumerate() {
        if letter.is_vowel() {
            if !cluster.is_empty() && cluster.last() != Some(&(i - 1)) {
                break; // only the first contiguous run
            }
            cluster.push(i);
        } else if !cluster.is_empty() {
            break;
        }
    }
    if cluster.len() > 1 {
        let first = cluster[0];
        let prev = first.checked_sub(1).map(|p| word[p].base);
        let consonant_glide = (prev == Some('q') && word[first].base == 'u')
            || (prev == Some('g') && word[first].base == 'i');
        if consonant_glide {
            cluster.remove(0);
        }
    }
    cluster
}

/// Where the tone mark goes (old style: hòa, thủy, khỏe; hoàn, toán).
fn tone_position(word: &[Letter]) -> Option<usize> {
    let cluster = vowel_cluster(word);
    if cluster.is_empty() {
        return None;
    }
    if let Some(&i) = cluster.iter().rev().find(|&&i| word[i].is_modified()) {
        return Some(i);
    }
    let has_final = cluster.last().is_some_and(|&last| last + 1 < word.len());
    Some(match cluster.len() {
        1 => cluster[0],
        2 if has_final => cluster[1],
        2 => cluster[0],
        _ => cluster[1],
    })
}

fn current_tone(word: &[Letter]) -> usize {
    word.iter().map(|l| l.tone).find(|&t| t != 0).unwrap_or(0)
}

/// Moves the word's tone mark to the correct vowel.
fn normalize_tone(word: &mut [Letter]) {
    let tone = current_tone(word);
    if tone == 0 {
        return;
    }
    for letter in word.iter_mut() {
        letter.tone = 0;
    }
    if let Some(pos) = tone_position(word) {
        word[pos].tone = tone;
    }
}

fn set_base(letter: &mut Letter, base: char) {
    letter.base = base;
}

/// Applies `key` to the end of `text` and returns the new text.
pub fn apply_key(text: &str, key: char) -> String {
    if !key.is_ascii_alphabetic() {
        let mut out = text.to_string();
        out.push(key);
        return out;
    }
    // Split off the last word (trailing letters).
    let chars: Vec<char> = text.chars().collect();
    let start = chars
        .iter()
        .rposition(|c| !c.is_alphabetic())
        .map_or(0, |p| p + 1);
    let prefix: String = chars[..start].iter().collect();
    let mut word: Vec<Letter> = chars[start..]
        .iter()
        .map(|&c| Letter::from_char(c))
        .collect();
    let k = key.to_ascii_lowercase();
    let key_upper = key.is_ascii_uppercase();
    let append = |word: &mut Vec<Letter>| {
        word.push(Letter {
            base: k,
            tone: 0,
            upper: key_upper,
        });
    };

    let has_vowel = word.iter().any(|l| l.is_vowel());

    if let Some(tone) = tone_of_key(k).filter(|_| has_vowel) {
        if current_tone(&word) == tone {
            for letter in word.iter_mut() {
                letter.tone = 0;
            }
            append(&mut word);
        } else if let Some(pos) = tone_position(&word) {
            for letter in word.iter_mut() {
                letter.tone = 0;
            }
            word[pos].tone = tone;
        } else {
            append(&mut word);
        }
    } else if k == 'z' && has_vowel && current_tone(&word) != 0 {
        for letter in word.iter_mut() {
            letter.tone = 0;
        }
    } else if matches!(k, 'a' | 'e' | 'o') {
        let cluster = vowel_cluster(&word);
        let target = cluster
            .iter()
            .rev()
            .copied()
            .find(|&i| word[i].plain() == k);
        match target {
            Some(i) => {
                let circumflex = match k {
                    'a' => 'â',
                    'e' => 'ê',
                    _ => 'ô',
                };
                if word[i].base == circumflex {
                    set_base(&mut word[i], k);
                    append(&mut word);
                } else {
                    set_base(&mut word[i], circumflex);
                }
            }
            None => append(&mut word),
        }
    } else if k == 'w' {
        let cluster = vowel_cluster(&word);
        let uo = cluster
            .windows(2)
            .find(|w| word[w[0]].plain() == 'u' && word[w[1]].plain() == 'o');
        if let Some(pair) = uo.filter(|p| !(word[p[0]].base == 'ư' && word[p[1]].base == 'ơ')) {
            let (u, o) = (pair[0], pair[1]);
            set_base(&mut word[u], 'ư');
            set_base(&mut word[o], 'ơ');
        } else if let Some(&i) = cluster
            .iter()
            .rev()
            .find(|&&i| matches!(word[i].plain(), 'a' | 'o' | 'u'))
        {
            let (plain, hooked) = match word[i].plain() {
                'a' => ('a', 'ă'),
                'o' => ('o', 'ơ'),
                _ => ('u', 'ư'),
            };
            if word[i].base == hooked {
                set_base(&mut word[i], plain);
                append(&mut word);
            } else {
                set_base(&mut word[i], hooked);
            }
        } else if !has_vowel {
            word.push(Letter {
                base: 'ư',
                tone: 0,
                upper: key_upper,
            });
        } else {
            append(&mut word);
        }
    } else if k == 'd' && word.first().is_some_and(|l| l.plain() == 'd') {
        if word[0].base == 'đ' {
            word[0].base = 'd';
            append(&mut word);
        } else {
            word[0].base = 'đ';
        }
    } else {
        append(&mut word);
    }

    normalize_tone(&mut word);
    let mut out = prefix;
    out.extend(word.iter().map(|l| l.to_char()));
    out
}

/// Types a whole string of keys (useful for tests and defaults).
pub fn type_keys(keys: &str) -> String {
    keys.chars()
        .fold(String::new(), |text, k| apply_key(&text, k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_name() {
        assert_eq!(type_keys("Laam Voo Traafn"), "Lâm Vô Trần");
        assert_eq!(type_keys("Laam Voo Traanf"), "Lâm Vô Trần");
    }

    #[test]
    fn common_names() {
        assert_eq!(type_keys("Nguyeenx Thij Huwowngf"), "Nguyễn Thị Hường");
        assert_eq!(type_keys("Nguyeexn"), "Nguyễn");
        assert_eq!(type_keys("Ddwcs"), "Đức");
        assert_eq!(type_keys("Tieeu Laan"), "Tiêu Lân");
        assert_eq!(type_keys("Phamj Thanh Tuaans"), "Phạm Thanh Tuấn");
        assert_eq!(type_keys("Hoaf"), "Hòa");
        assert_eq!(type_keys("Hoafn"), "Hoàn");
        assert_eq!(type_keys("Thuyr"), "Thủy");
        assert_eq!(type_keys("khoer"), "khỏe");
        assert_eq!(type_keys("nguowif"), "người");
        assert_eq!(type_keys("quas"), "quá");
        assert_eq!(type_keys("giuwxa"), "giữa");
        assert_eq!(type_keys("gif"), "gì");
    }

    #[test]
    fn undo_by_repeating() {
        assert_eq!(type_keys("ass"), "as");
        assert_eq!(type_keys("aaa"), "aa");
        assert_eq!(type_keys("aww"), "aw");
        assert_eq!(type_keys("ddd"), "dd");
        assert_eq!(type_keys("asz"), "a");
    }

    #[test]
    fn w_alone_and_plain_text() {
        assert_eq!(type_keys("w"), "ư");
        assert_eq!(type_keys("tw"), "tư");
        assert_eq!(type_keys("Mai-Lan"), "Mai-Lan");
        assert_eq!(type_keys("s"), "s");
    }

    #[test]
    fn keeps_unicode_input_untouched() {
        assert_eq!(apply_key("Lâm Vô Trầ", 'n'), "Lâm Vô Trần");
        assert_eq!(apply_key("Lâm ", 'V'), "Lâm V");
    }
}
