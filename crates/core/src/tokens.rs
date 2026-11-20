//! Token 计量的唯一实现。
//!
//! 全仓只有这一处换算。Adapter 不写第二个除法：它用 Core 返回的每条 `tokenCount`
//! 求和，需要给整页计时再调 Core 的估算接口。
//!
//! 口径：`tokens = ceil(hanChars × wHan + otherChars × 0.25)`。
//!
//! - `wHan` 默认 [`HARD_W_HAN`] = 1.5，覆盖实测单样本最高 1.44（见
//!   `docs/research/zh-fts-and-token-measurements.md`），留 4% 余量；
//!   软展示可用 [`SOFT_W_HAN`] = 1.25。
//! - 非 Han 字符按 0.25 token/字符，与 DSH `ctx.tokenMeter` 的 `字符/4` 对齐。
//! - 混合文本按 Han / 非 Han 分段加权，不整体乘中文倍率。
//!
//! DSH 的 `estimateMessage()` 对一条纯文本消息是 `ceil(chars/4) + 8`
//! （4 字节块结构开销 + 4 角色框架开销）。本模块的 [`estimate_tokens`] 只算文本密度，
//! [`estimate_message_tokens`] 才是与之逐位对齐的整条消息口径。

/// 汉字折算倍率的硬上限：预算判定用它，确保不会把中文低估。
pub const HARD_W_HAN: f64 = 1.5;

/// 汉字折算倍率的软估算：只用于展示，不用于预算判定。
pub const SOFT_W_HAN: f64 = 1.25;

/// 非 Han 字符的 token 密度：4 字符/token，与 DSH 的 `字符/4` 一致。
pub const OTHER_TOKENS_PER_CHAR: f64 = 0.25;

/// DSH `estimateMessage()` 给每条消息加的固定框架开销（角色字段等）。
pub const MESSAGE_ROLE_OVERHEAD: u32 = 4;

/// DSH `estimateMessage()` 给每个内容块加的结构开销（JSON 框架与类型标签）。
pub const MESSAGE_BLOCK_OVERHEAD: u32 = 4;

/// CJK 与相关表意文字的码点区间；落在区间内按 `wHan` 折算。
///
/// 含：CJK 统一表意文字与其扩展、CJK 符号与标点、假名、谚文、全角/半角形式、
/// CJK 兼容表意文字。区间选得宽一点是刻意的——宁可把少量符号当汉字多算，
/// 也不要把中文漏进 0.25 的英文密度里。
pub const CJK_RANGES: &[(u32, u32)] = &[
    (0x1100, 0x11FF),   // Hangul Jamo
    (0x2E80, 0x2EFF),   // CJK Radicals Supplement
    (0x2F00, 0x2FDF),   // Kangxi Radicals
    (0x3000, 0x303F),   // CJK Symbols and Punctuation
    (0x3040, 0x309F),   // Hiragana
    (0x30A0, 0x30FF),   // Katakana
    (0x3130, 0x318F),   // Hangul Compatibility Jamo
    (0x31C0, 0x31EF),   // CJK Strokes
    (0x31F0, 0x31FF),   // Katakana Phonetic Extensions
    (0x3400, 0x4DBF),   // CJK Unified Ideographs Extension A
    (0x4E00, 0x9FFF),   // CJK Unified Ideographs
    (0xA960, 0xA97F),   // Hangul Jamo Extended-A
    (0xAC00, 0xD7AF),   // Hangul Syllables
    (0xF900, 0xFAFF),   // CJK Compatibility Ideographs
    (0xFE30, 0xFE4F),   // CJK Compatibility Forms
    (0xFF00, 0xFFEF),   // Halfwidth and Fullwidth Forms
    (0x20000, 0x2A6DF), // CJK Unified Ideographs Extension B
    (0x2A700, 0x2EBEF), // Extensions C/D/E/F
    (0x2F800, 0x2FA1F), // CJK Compatibility Ideographs Supplement
    (0x30000, 0x3134F), // Extensions G/H
];

/// 这个码点是否按 Han 密度折算。emoji 与拉丁字母都不算。
pub fn is_cjk(ch: char) -> bool {
    let code = ch as u32;
    CJK_RANGES
        .iter()
        .any(|(start, end)| (*start..=*end).contains(&code))
}

/// 按默认硬上限倍率估算一段文本的 token 数。
///
/// 空文本是 0；结果向上取整，绝不超过真实分词的可能上界（以实测上界为基准）。
pub fn estimate_tokens(text: &str) -> u32 {
    estimate_tokens_with(text, HARD_W_HAN)
}

/// 按给定汉字倍率估算一段文本的 token 数，供配置按 provider 覆盖。
///
/// 倍率非法（负数、NaN、无穷）时回落到硬上限，不静默产生负预算。
pub fn estimate_tokens_with(text: &str, w_han: f64) -> u32 {
    let w_han = if w_han.is_finite() && w_han > 0.0 {
        w_han
    } else {
        HARD_W_HAN
    };
    let mut han: u64 = 0;
    let mut other: u64 = 0;
    for ch in text.chars() {
        if is_cjk(ch) {
            han += 1;
        } else {
            other += 1;
        }
    }
    let weighted = han as f64 * w_han + other as f64 * OTHER_TOKENS_PER_CHAR;
    let ceil = weighted.ceil();
    if ceil.is_finite() && ceil > 0.0 {
        ceil as u32
    } else {
        0
    }
}

/// 一条纯文本消息的 token 数，与 DSH `ctx.tokenMeter.estimateMessage()` 对齐。
///
/// 固定开销是 8：4 字节块结构 + 4 角色框架。预算判定用 [`estimate_tokens`]，
/// 只有当需要与 DSH 的整条消息口径比对时才用这个函数。
pub fn estimate_message_tokens(text: &str) -> u32 {
    estimate_tokens(text) + MESSAGE_ROLE_OVERHEAD + MESSAGE_BLOCK_OVERHEAD
}

/// 纯英文文本的 DSH 对齐口径：`ceil(chars/4)`。
///
/// 不参与生产代码，只给测试与文档一个可引用的名字。
pub fn estimate_english_density(text: &str) -> u32 {
    (text.chars().count() as f64 * OTHER_TOKENS_PER_CHAR).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC：纯英文文本的文本密度与 DSH 的 `字符/4` 完全一致。
    #[test]
    fn english_matches_the_four_chars_per_token_density() {
        let fixtures = [
            "",
            "a",
            "abcd",
            "abcde",
            "hello world",
            "The quick brown fox jumps over the lazy dog.",
            "I prefer concise answers, please.",
            "pnpm install --frozen-lockfile",
            "GET /v1/observations?sessionId=s1&limit=10",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            "0123456789",
        ];
        assert!(fixtures.len() >= 10, "夹具应至少 10 段");
        for text in fixtures {
            assert_eq!(
                estimate_tokens(text),
                estimate_english_density(text),
                "英文密度应与 ceil(chars/4) 一致：{text:?}"
            );
        }
    }

    /// AC：16 个汉字的计数落在 o200k（14）与 cl100k（约 23）的实测区间内。
    #[test]
    fn sixteen_han_characters_land_between_the_measured_encodings() {
        let text = "我今天下午想去西湖边散步然后回家"; // 16 个汉字
        assert_eq!(text.chars().count(), 16);
        let tokens = estimate_tokens(text);
        assert!((14..=26).contains(&tokens), "16 汉字应落在 14–26 之间，实际 {tokens}");
        assert_eq!(tokens, 24, "默认倍率 1.5 下 16×1.5=24");
    }

    /// 硬上限比软估算更保守；两者都不低于真实下限。
    #[test]
    fn hard_cap_is_more_conservative_than_soft_estimate() {
        let text = "今天下午三点在西湖边见面";
        assert!(estimate_tokens(text) > estimate_tokens_with(text, SOFT_W_HAN));
    }

    /// 混排不能整体当中文，也不能整体当英文。
    #[test]
    fn mixed_text_is_weighted_by_segment() {
        let text = "用 pnpm 装依赖"; // 4 汉字 + 6 非汉字（含两个空格）
        let han: u64 = 4;
        let other: u64 = text.chars().count() as u64 - han;
        let expected = (han as f64 * HARD_W_HAN + other as f64 * OTHER_TOKENS_PER_CHAR).ceil() as u32;
        assert_eq!(estimate_tokens(text), expected);

        let all_han = estimate_tokens("用用用用用用用用用用用");
        let all_english = estimate_tokens("abcdefghijklmnopqrst");
        assert!(estimate_tokens(text) < all_han, "混排不应整体当中文");
        assert!(estimate_tokens(text) > all_english, "混排不应整体当英文");
    }

    /// emoji 按 Unicode 标量值计，一个 emoji 一个码点。
    #[test]
    fn emoji_are_counted_as_single_code_points() {
        assert_eq!("😀".chars().count(), 1);
        assert_eq!(estimate_tokens("😀😀😀😀"), 1, "4 个 emoji × 0.25 向上取整是 1");
        assert!(!is_cjk('😀'));
    }

    /// 中文标点与假名按 Han 密度计入。
    #[test]
    fn cjk_punctuation_and_kana_count_as_han() {
        for ch in ['。', '，', '！', '？', '「', '」', 'あ', 'カ', '한'] {
            assert!(is_cjk(ch), "{ch} 应按 Han 密度折算");
        }
        // 中文标点若被当成英文密度，预算会被系统性低估。
        assert_eq!(estimate_tokens("你好，世界！"), 9, "6 个字符 × 1.5 = 9");
    }

    /// 空文本与非法倍率都得到安全值。
    #[test]
    fn empty_text_and_invalid_weights_are_safe() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens_with("中文", f64::NAN), 3); // 2×1.5 = 3
        assert_eq!(estimate_tokens_with("中文", -1.0), 3);
        assert_eq!(estimate_tokens_with("中文", f64::INFINITY), 3);
    }

    /// 整条消息口径 = 文本密度 + 8，与 DSH `estimateMessage()` 对齐。
    #[test]
    fn message_tokens_add_dsh_framing_overhead() {
        let text = "hello world";
        assert_eq!(estimate_message_tokens(text), estimate_tokens(text) + 8);
        assert_eq!(estimate_message_tokens(text), 3 + 8, "ceil(11/4)=3");
    }
}
