use regex::Regex;

use crate::aozora_parser::tokenizer::CommandToken;

const ZERO_TO_NINE: [char; 10] = ['０', '１', '２', '３', '４', '５', '６', '７', '８', '９'];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MidashiSize {
    Large,
    Middle,
    Small,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MidashiType {
    Normal,
    Dogyo,
    Mado,
}

/// 傍点を表します．詳細は以下のURLを参照してください．
///
/// https://www.aozora.gr.jp/annotation/emphasis.html#boten_chuki
enum Bouten {
    Sirogoma,
    BlackCircle,
    WhiteCircle,
    BlackTriangle,
    WhiteTriangle,
    DoubleCircle,
    Hebinome,
    Cross,
}

/// 傍線を表します．詳細は以下のURLを参照してください．
///
/// https://www.aozora.gr.jp/annotation/emphasis.html#bosen_chuki
enum Bousen {
    Bousen,
    Double,
    Chain,
    Dashed,
    Wavy,
}

/// is_inlineが真である場合，同行見出しとして解釈されます．
///
/// 詳細は以下のURLを参照してください．
///
/// https://www.aozora.gr.jp/annotation/heading.html#dogyo_midashi
#[derive(Debug, PartialEq, Clone)]
pub struct Midashi {
    pub size: MidashiSize,
    pub kind: MidashiType,
}

/// 字下げ，地付き，字寄せを表現する構造体です．
///
/// # 字下げ
/// Commandで呼び出されている場合，一行字下げとなります．
/// 以下のURLを参照してください。
///
/// https://www.aozora.gr.jp/annotation/layout_2.html#ichigyo
///
/// # ブロック字下げ
/// CommandBeginとCommandEndに挟まれている場合，
/// ブロック字下げとなります．以下のURLを参照してください．
///
/// https://www.aozora.gr.jp/annotation/layout_2.html#jisage
///
/// # 地付き
/// 地付きはis_upperが偽，spaceが0として解釈されます．
/// 地付きの詳細は以下のURLを参照してください．
///
/// https://www.aozora.gr.jp/annotation/layout_2.html#chitsuki
struct Alignment {
    is_upper: bool,
    space: usize,
}

enum CommandBegin {
    // Other
    Midashi(Midashi),
    Alignment(Alignment),

    // Emphasis
    Bouten(Bouten),
    Bousen(Bousen),
    Bold,
    Italic,

    // Block
    /// 罫囲みを表します．詳細は以下のURLを参照してください．
    ///
    /// https://www.aozora.gr.jp/annotation/etc.html#keigakomi
    Kakomikei,
    /// 横組みを表します．詳細は以下のURLを参照してください．
    ///
    /// https://www.aozora.gr.jp/annotation/etc.html#yokogumi
    Yokogumi,
    /// 字詰めを表します．詳細は以下のURLを参照してください．
    ///
    /// https://www.aozora.gr.jp/annotation/etc.html#jizume
    Jitsume(usize),
}

enum CommandEnd {
    // Other
    Midashi(Midashi),
    Alignment,

    // Emphasis
    Bouten,
    Bousen,
    Bold,
    Italic,

    // Block
    Kakomikei,
    Yokogumi,
    Jitsume,
}

enum SingleCommand {
    // Other
    Midashi((Midashi, String)),
    Alignment(Alignment),

    // Break
    Kaicho,
    Kaimihiraki,
    Kaipage,
    Kaidan,

    // Emphasis
    Bouten((Bouten, String)),
    Bousen((Bousen, String)),
    Bold(String),
    Italic(String),
}

enum Command {
    CommandBegin(CommandBegin),
    SingleCommand(SingleCommand),
    CommandEnd(CommandEnd),
}

fn full_width_digit_to_u32(input: &str) -> Option<u32> {
    let smallified: String = input
        .chars()
        .map(|c| match c {
            '０'..='９' => char::from_u32(c as u32 - '０' as u32 + '0' as u32).unwrap(),
            _ => c,
        })
        .collect();
    smallified.parse::<u32>().ok()
}

pub fn parse_command(commands: CommandToken) -> Option<Command> {
    let s = commands.content.as_str();

    // Regex for references (e.g. 「...」は...見出し)
    let re_ref = Regex::new(r"^「(?P<content>.+?)」は(?P<type>同行|窓)?(?P<size>大|中|小)見出し$").unwrap();

    // Regex for block begin (e.g. ここから...見出し, or simple ...見出し)
    let re_begin = Regex::new(r"^(?:ここから)?(?P<type>同行|窓)?(?P<size>大|中|小)見出し$").unwrap();

    // Regex for block end (e.g. ここで...見出し終わり, or ...見出し終わり)
    let re_end = Regex::new(r"^(?:ここで)?(?P<type>同行|窓)?(?P<size>大|中|小)見出し終わり$").unwrap();

    // Regex for jisage (e.g. １０字下げ)
    let re_jisage = Regex::new(r"^(?P<num>[１２３４５６７８９０]+)字下げ$").unwrap();
    // Regex for block jisage begin (e.g. ここから１０字下げ)
    let re_jisage_begin = Regex::new(r"^ここから(?P<num>[１２３４５６７８９０]+)字下げ$").unwrap();

    if let Some(caps) = re_ref.captures(s) {
        let content = caps.name("content").unwrap().as_str().to_string();
        let size = match caps.name("size").unwrap().as_str() {
            "大" => MidashiSize::Large,
            "中" => MidashiSize::Middle,
            "小" => MidashiSize::Small,
            _ => unreachable!(),
        };
        let kind = match caps.name("type").map(|m| m.as_str()) {
            Some("同行") => MidashiType::Dogyo,
            Some("窓") => MidashiType::Mado,
            _ => MidashiType::Normal,
        };
        return Some(Command::SingleCommand(SingleCommand::Midashi((
            Midashi { size, kind },
            content,
        ))));
    } else if let Some(caps) = re_begin.captures(s) {
        let size = match caps.name("size").unwrap().as_str() {
            "大" => MidashiSize::Large,
            "中" => MidashiSize::Middle,
            "小" => MidashiSize::Small,
            _ => unreachable!(),
        };
        let kind = match caps.name("type").map(|m| m.as_str()) {
            Some("同行") => MidashiType::Dogyo,
            Some("窓") => MidashiType::Mado,
            _ => MidashiType::Normal,
        };
        return Some(Command::CommandBegin(CommandBegin::Midashi(Midashi {
            size,
            kind,
        })));
    } else if let Some(caps) = re_end.captures(s) {
        let size = match caps.name("size").unwrap().as_str() {
            "大" => MidashiSize::Large,
            "中" => MidashiSize::Middle,
            "小" => MidashiSize::Small,
            _ => unreachable!(),
        };
        let kind = match caps.name("type").map(|m| m.as_str()) {
            Some("同行") => MidashiType::Dogyo,
            Some("窓") => MidashiType::Mado,
            _ => MidashiType::Normal,
        };
        return Some(Command::CommandEnd(CommandEnd::Midashi(Midashi {
            size,
            kind,
        })));
    } else if let Some(caps) = re_jisage.captures(s) {
        let num_str = caps.name("num").unwrap().as_str();
        if let Some(n) = full_width_digit_to_u32(num_str) {
            return Some(Command::SingleCommand(SingleCommand::Alignment(
                Alignment {
                    is_upper: true,
                    space: n as usize,
                },
            )));
        }
    } else if let Some(caps) = re_jisage_begin.captures(s) {
        let num_str = caps.name("num").unwrap().as_str();
        if let Some(n) = full_width_digit_to_u32(num_str) {
            return Some(Command::CommandBegin(CommandBegin::Alignment(Alignment {
                is_upper: true,
                space: n as usize,
            })));
        }
    }

    match s {
        "改丁" => Some(Command::SingleCommand(SingleCommand::Kaicho)),
        "改ページ" => Some(Command::SingleCommand(SingleCommand::Kaipage)),
        "改見開き" => Some(Command::SingleCommand(SingleCommand::Kaimihiraki)),
        "改段" => Some(Command::SingleCommand(SingleCommand::Kaidan)),
        "ここで字下げ終わり" => Some(Command::CommandEnd(CommandEnd::Alignment)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aozora_parser::tokenizer::CommandToken;

    #[test]
    fn test_midashi_ref() {
        let token = CommandToken {
            content: "「独り寝の別れ」は大見出し".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::SingleCommand(SingleCommand::Midashi((m, c))) => {
                assert_eq!(m.size, MidashiSize::Large);
                assert_eq!(m.kind, MidashiType::Normal);
                assert_eq!(c, "独り寝の別れ");
            }
            _ => panic!("Expected Midashi SingleCommand"),
        }

        let token = CommandToken {
            content: "「入藏を思ひ立ツた原因」は同行中見出し".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::SingleCommand(SingleCommand::Midashi((m, c))) => {
                assert_eq!(m.size, MidashiSize::Middle);
                assert_eq!(m.kind, MidashiType::Dogyo);
                assert_eq!(c, "入藏を思ひ立ツた原因");
            }
            _ => panic!("Expected Midashi SingleCommand"),
        }
        
        let token = CommandToken {
            content: "「青空文庫」は窓中見出し".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::SingleCommand(SingleCommand::Midashi((m, c))) => {
                assert_eq!(m.size, MidashiSize::Middle);
                assert_eq!(m.kind, MidashiType::Mado);
                assert_eq!(c, "青空文庫");
            }
            _ => panic!("Expected Midashi SingleCommand"),
        }
    }

    #[test]
    fn test_midashi_begin() {
        let token = CommandToken {
            content: "大見出し".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::CommandBegin(CommandBegin::Midashi(m)) => {
                assert_eq!(m.size, MidashiSize::Large);
                assert_eq!(m.kind, MidashiType::Normal);
            }
            _ => panic!("Expected Midashi CommandBegin"),
        }

        let token = CommandToken {
            content: "同行小見出し".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::CommandBegin(CommandBegin::Midashi(m)) => {
                assert_eq!(m.size, MidashiSize::Small);
                assert_eq!(m.kind, MidashiType::Dogyo);
            }
            _ => panic!("Expected Midashi CommandBegin"),
        }

        let token = CommandToken {
            content: "ここから窓中見出し".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::CommandBegin(CommandBegin::Midashi(m)) => {
                assert_eq!(m.size, MidashiSize::Middle);
                assert_eq!(m.kind, MidashiType::Mado);
            }
            _ => panic!("Expected Midashi CommandBegin"),
        }
    }

    #[test]
    fn test_midashi_end() {
        let token = CommandToken {
            content: "大見出し終わり".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::CommandEnd(CommandEnd::Midashi(m)) => {
                assert_eq!(m.size, MidashiSize::Large);
                assert_eq!(m.kind, MidashiType::Normal);
            }
            _ => panic!("Expected Midashi CommandEnd"),
        }

        let token = CommandToken {
            content: "ここで窓中見出し終わり".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::CommandEnd(CommandEnd::Midashi(m)) => {
                assert_eq!(m.size, MidashiSize::Middle);
                assert_eq!(m.kind, MidashiType::Mado);
            }
            _ => panic!("Expected Midashi CommandEnd"),
        }
    }

    #[test]
    fn test_jisage() {
        let token = CommandToken {
            content: "１字下げ".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::SingleCommand(SingleCommand::Alignment(a)) => {
                assert_eq!(a.space, 1);
                assert!(a.is_upper);
            }
            _ => panic!("Expected Alignment SingleCommand"),
        }

        let token = CommandToken {
            content: "ここから１０字下げ".to_string(),
        };
        let cmd = parse_command(token).unwrap();
        match cmd {
            Command::CommandBegin(CommandBegin::Alignment(a)) => {
                assert_eq!(a.space, 10);
                assert!(a.is_upper);
            }
            _ => panic!("Expected Alignment CommandBegin"),
        }
    }
}
