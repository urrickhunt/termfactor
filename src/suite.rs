#[derive(Clone, Copy)]
pub enum Display {
    Lines(&'static [&'static str]),
    Gradient,
}

#[derive(Clone, Copy)]
pub struct Fallback {
    pub display: Display,
    pub prompt: &'static str,
}

#[derive(Clone, Copy)]
pub struct Check {
    pub header: &'static str,
    pub display: Display,
    pub prompt: &'static str,
    pub fallback: Option<Fallback>,
}

pub const POINTS_PER_CHECK: u16 = 5;
pub const CHECK_COUNT: u16 = 24;
pub const CHECKS: [Check; CHECK_COUNT as usize] = [
    check(
        "truecolor",
        Display::Lines(&["\x1b[38;2;255;100;0mtruecolor\x1b[0m"]),
        "is the word 'truecolor' displayed in \x1b[38;2;255;100;0msafety orange\x1b[0m?",
    ),
    check(
        "truecolor",
        Display::Gradient,
        "is a \x1b[0;31mred\x1b[0m/\x1b[0;32mgreen\x1b[0m/\x1b[0;34mblue\x1b[0m color gradient displayed?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[1mbold\x1b[22m"]),
        "is the text bold?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[2mdim\x1b[22m"]),
        "is the text dim?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[3mitalic\x1b[23m"]),
        "is the text italic?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[4munderline\x1b[24m"]),
        "is the text underlined?",
    ),
    check_with_fallback(
        "text decorations",
        Display::Lines(&["\x1b[4:2mdouble underline\x1b[24m"]),
        "is the text double underlined?",
        Fallback {
            display: Display::Lines(&["\x1b[21malt double underline\x1b[24m"]),
            prompt: "is the text double underlined now?",
        },
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[4:3mcurly underline\x1b[24m"]),
        "is the text curly underlined?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[4:4mdotted underline\x1b[24m"]),
        "is the text dotted underlined?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[4:5mdashed underline\x1b[24m"]),
        "is the text dashed underlined?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[4;58:5:212mcolored underline\x1b[59;24m"]),
        "is the text pink underlined?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[5mblink\x1b[25m"]),
        "is the text blinking?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[7mreverse\x1b[27m"]),
        "is the text reversed?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[8minvisible\x1b[28m <- invisible (but copy-pasteable)"]),
        "is the text invisible but still copy-pasteable?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[9mstrikethrough\x1b[29m"]),
        "is the text strikethrough?",
    ),
    check(
        "text decorations",
        Display::Lines(&["\x1b[53moverline\x1b[55m"]),
        "is the text overlined?",
    ),
    check(
        "unicode",
        Display::Lines(&["Δ Й ह क あ 葉 ᄀ ቐ ๗ ഷ"]),
        "are all 10 unicode symbols displayed?",
    ),
    check(
        "emojis",
        Display::Lines(&["👻💜🧛"]),
        "are emojis 👻💜🧛 displayed?",
    ),
    check(
        "bidi aware ('ע' & 'ا' symbols should be on the right side)",
        Display::Lines(&["עברית"]),
        "is the 'ע' symbol on the right side?",
    ),
    check(
        "bidi aware ('ע' & 'ا' symbols should be on the right side)",
        Display::Lines(&["العربية"]),
        "is the 'ا' symbol on the right side?",
    ),
    check(
        "osc 8 hyperlinks",
        Display::Lines(&[
            "\x1b]8;;https://github.com/Alhadis/OSC8-Adoption\x1b\\🔗osc 8 terminal adoption\x1b]8;;\x1b\\",
        ]),
        "is a clickable hyperlink displayed (use modifier keys ctrl+shift if needed)?",
    ),
    check(
        "osc 9 notifications",
        Display::Lines(&["\x1b]9;✔osc 9 notifications\x07\x1b[5m✔check notifications\x1b[25m"]),
        "did you get the osc 9 notification?",
    ),
    check(
        "osc 777 notifications",
        Display::Lines(&["\x1b]777;notify;✔osc;777 notifications\x1b\\\x1b[5m✔check notifications\x1b[25m"]),
        "did you get the osc 777 notification?",
    ),
    check(
        "sixel graphics",
        Display::Lines(&[
            "\x1bP0;0;0q\"1;1;64;64#0;2;0;0;0#1;2;100;100;100#1~{wo_!11?@FN^!34~^NB\
             @?_ow{~$#0?BFN^!11~}wo_!34?_o{}~^NFB-#1!5~}{o_!12?BF^!25~^NB@??ow{!6~$#0!5?\
             @BN^!12~{w_!25?_o{}~~NFB-#1!10~}w_!12?@BN^!15~^NFB@?_w{}!10~$#0!10?@F^!12~}\
             {o_!15?_ow{}~^FB@-#1!14~}{o_!11?@BF^!7~^FB??_ow}!15~$#0!14?@BN^!11~}{w_!7?_\
             w{~~^NF@-#1!18~}{wo!11?_r^FB@??ow}!20~$#0!18?@BFN!11~^K_w{}~~NF@-#1!23~M!4?_\
             oWMF@!6?BN^!21~$#0!23?p!4~^Nfpw}!6~{o_-#1!18~^NB@?_ow{}~wo!12?@BFN!17~$#0!18?_\
             o{}~^NFB@?FN!12~}{wo-#1!13~^NB@??_w{}!9~}{w_!12?BFN^!12~$#0!13?_o{}~~^FB@!9?\
             @BF^!12~{wo_-#1!8~^NFB@?_w{}!19~{wo_!11?@BN^!8~$#0!8?_ow{}~^FB@!19?BFN^!11~}\
             {o_-#1!4~^NB@?_ow{!28~}{o_!12?BF^!4~$#0!4?_o{}~^NFB!28?@BN^!12~{w_-#1NB@???GM\
             !38NMG!13?@BN$#0?KMNNNF@!38?@F!13NMK-\x1b\\",
        ]),
        "is a sixel graphic displayed?",
    ),
];
pub const MAX_SCORE: u16 = CHECK_COUNT * POINTS_PER_CHECK;

const fn check(header: &'static str, display: Display, prompt: &'static str) -> Check {
    Check {
        header,
        display,
        prompt,
        fallback: None,
    }
}

const fn check_with_fallback(
    header: &'static str,
    display: Display,
    prompt: &'static str,
    fallback: Fallback,
) -> Check {
    Check {
        header,
        display,
        prompt,
        fallback: Some(fallback),
    }
}

#[cfg(test)]
mod tests {
    use super::{CHECKS, CHECK_COUNT, MAX_SCORE, POINTS_PER_CHECK};

    #[test]
    fn catalog_constants_stay_in_sync() {
        assert_eq!(CHECKS.len(), CHECK_COUNT as usize);
        assert_eq!(MAX_SCORE, CHECK_COUNT * POINTS_PER_CHECK);
    }
}
