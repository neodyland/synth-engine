use regex::{Captures, Regex};
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

mod all_csv;

fn capture<'a>(cap: &'a Captures, name: &str) -> Option<&'a str> {
    cap.name(name).map(|s| s.as_str())
}

fn yomi_chr(c: char) -> Option<&'static str> {
    Some(match c {
        '0' => "ゼロ",
        '1' => "イチ",
        '2' => "ニー",
        '3' => "サン",
        '4' => "ヨン",
        '5' => "ゴー",
        '6' => "ロク",
        '7' => "ナナ",
        '8' => "ハチ",
        '9' => "キュウ",
        _ => return None,
    })
}
fn yomi_chr_short(c: char) -> Option<&'static str> {
    Some(match c {
        '0' => "ゼロ",
        '1' => "イチ",
        '2' => "ニ",
        '3' => "サン",
        '4' => "ヨン",
        '5' => "ゴ",
        '6' => "ロク",
        '7' => "ナナ",
        '8' => "ハチ",
        '9' => "キュウ",
        _ => return None,
    })
}

fn try_phone(cap: &Captures) -> Option<String> {
    let country =
        capture(cap, "country").map(|c| c.chars().filter_map(yomi_chr).collect::<String>());
    let first = capture(cap, "first")?
        .chars()
        .filter_map(yomi_chr)
        .collect::<String>();
    let second = capture(cap, "second")?
        .chars()
        .filter_map(yomi_chr)
        .collect::<String>();
    let third = capture(cap, "third")?
        .chars()
        .filter_map(yomi_chr)
        .collect::<String>();
    let country = country.map(|c| format!("{c},")).unwrap_or_default();
    Some(format!("{}{first},{second},{third}.", country))
}

fn try_address(cap: &Captures) -> Option<String> {
    let first = capture(cap, "first")?
        .chars()
        .filter_map(yomi_chr)
        .collect::<String>();
    let second = capture(cap, "second")?
        .chars()
        .filter_map(yomi_chr)
        .collect::<String>();
    Some(format!("{first},{second}."))
}

fn try_justnumber(s: &str) -> Option<String> {
    let mut r = Vec::new();
    for (i, a) in s
        .chars()
        .filter(|c| *c != ',')
        .rev()
        .collect::<Vec<_>>()
        .chunks(4)
        .enumerate()
    {
        let post = match i {
            1 => "マン",              //万
            2 => "オク",              //億
            3 => "チョウ",            //兆
            4 => "ケイ",              //京
            5 => "ガイ",              //垓
            6 => "ジョ",              //杼
            7 => "ジョウ",            //穣
            8 => "コウ",              //溝
            9 => "カン",              //澗
            10 => "セイ",             //正
            11 => "サイ",             //載
            12 => "ゴク",             //極
            13 => "ゴウガシャ",       //恒河沙
            14 => "アソウギ",         //阿僧祇
            15 => "ナユタ",           //那由他
            16 => "フカシギ",         //不可思議
            17 => "ムリョウタイスウ", //無量大数
            _ => "",
        };
        let mut y = String::new();
        if let Some(a) = a.get(3)
            && *a != '0'
        {
            if *a != '1' {
                y.push_str(yomi_chr_short(*a)?);
            }
            y.push_str("セン");
        }
        if let Some(a) = a.get(2)
            && *a != '0'
        {
            if *a != '1' {
                y.push_str(yomi_chr_short(*a)?);
            }
            y.push_str("ヒャク");
        }
        if let Some(a) = a.get(1)
            && *a != '0'
        {
            if *a != '1' {
                y.push_str(yomi_chr_short(*a)?);
            }
            y.push_str("ジュウ");
        }
        if let Some(a) = a.first()
            && (y.is_empty() || *a != '0')
        {
            y.push_str(yomi_chr_short(*a)?);
        }
        r.push(format!("{y}{post}"))
    }
    r.reverse();
    Some(r.join(""))
}

fn try_number(cap: &Captures) -> Option<String> {
    let up = try_justnumber(capture(cap, "up").or(capture(cap, "downup")).unwrap_or("0"))?;
    let down = capture(cap, "down")
        .map(|c| format!("テン{}", c.chars().filter_map(yomi_chr).collect::<String>()))
        .unwrap_or_default();
    Some(format!("{up}{down}."))
}

fn special_time_half(d: String) -> String {
    match d.as_str() {
        "サンジュップン" => "ハン".to_string(),
        _ => d,
    }
}

fn special_time(d: String) -> String {
    match d.as_str() {
        d if d.ends_with("イチ") => {
            format!(
                "{}イップン",
                d.chars().take(d.chars().count() - 2).collect::<String>()
            )
        }
        d if d.ends_with("ロク") => {
            format!(
                "{}ロップン",
                d.chars().take(d.chars().count() - 2).collect::<String>()
            )
        }
        d if d.ends_with("ハチ") => {
            format!(
                "{}ハップン",
                d.chars().take(d.chars().count() - 2).collect::<String>()
            )
        }
        d if d.ends_with("ジュウ") => {
            format!(
                "{}ジュップン",
                d.chars().take(d.chars().count() - 3).collect::<String>()
            )
        }
        _ => format!("{d}フン"),
    }
}

fn try_time(cap: &Captures) -> Option<String> {
    let a = capture(cap, "a")?;
    let a = try_justnumber(a).unwrap_or(a.to_string());
    let b = capture(cap, "b")?;
    let b = try_justnumber(b).unwrap_or(b.to_string());
    let b = special_time(b);
    Some(
        if let Some(c) = capture(cap, "c").map(|c| try_justnumber(c).unwrap_or(c.to_string())) {
            format!("{a}ジ{b}{c}ビョウ")
        } else {
            format!("{a}ジ{}", special_time_half(b))
        },
    )
}

fn special_date(d: String) -> String {
    d.replace("ガツイチニチ", "ガツツイタチ")
        .replace("ガツニニチ", "ガツフツカ")
        .replace("ガツサンニチ", "ガツミッカ")
        .replace("ガツヨンニチ", "ガツヨッカ")
        .replace("ガツゴニチ", "ガツイツカ")
        .replace("ガツロクニチ", "ガツムイカ")
        .replace("ガツナナニチ", "ガツナノカ")
        .replace("ガツハチニチ", "ガツヨウカ")
        .replace("ガツキュウニチ", "ガツココノカ")
        .replace("ガツジュウニチ", "ガツトオカ")
        .replace("ガツニジュウニチ", "ガツハツカ")
}

fn try_date(cap: &Captures) -> Option<String> {
    let a = capture(cap, "b")?;
    let a = try_justnumber(a).unwrap_or(a.to_string());
    let b = capture(cap, "c")?;
    let b = try_justnumber(b).unwrap_or(b.to_string());
    Some(special_date(if let Some(c) = capture(cap, "a") {
        let c = try_justnumber(c).unwrap_or(c.to_string());
        format!("{c}ネン{a}ガツ{b}ニチ")
    } else {
        format!("{a}ガツ{b}ニチ")
    }))
}

struct NumberYomi {
    re_phone: Regex,
    re_address: Regex,
    re_number: Regex,
    re_time: Regex,
    re_date: Regex,
}

impl NumberYomi {
    pub fn new() -> anyhow::Result<Self> {
        let re_time =
            Regex::new(r"(?P<a>[0-9]{1,2})[:時h](?P<b>[0-9]{1,2})([:分m](?P<c>[0-9]{1,2})[秒s])?")?;
        let re_date =
            Regex::new(r"((?P<a>[0-9]{1,5})[/年])?(?P<b>[0-9]{1,5})[/月](?P<c>[0-9]{1,5})日?")?;
        let re_phone = Regex::new(
            r"(\+(?P<country>[0-9-]{1,5})) +?(?P<first>[0-9]{1,5})-(?P<second>[0-9]{1,5})-(?P<third>[0-9]{1,5})",
        )?;
        let re_address = Regex::new(r"(?P<first>[0-9]{3})-(?P<second>[0-9]{4})")?;
        let re_number = Regex::new(r"(?P<up>[0-9,]+)|((?P<downup>[0-9,]*)\.(?P<down>[0-9]+))")?;
        Ok(Self {
            re_phone,
            re_address,
            re_number,
            re_time,
            re_date,
        })
    }
    pub fn to_yomi(&self, s: &str) -> String {
        let s = self.re_date.replace_all(s, |cap: &Captures| {
            try_date(cap).unwrap_or(cap.get_match().as_str().to_string())
        });
        let s = self.re_time.replace_all(&s, |cap: &Captures| {
            try_time(cap).unwrap_or(cap.get_match().as_str().to_string())
        });
        let s = self.re_phone.replace_all(&s, |cap: &Captures| {
            try_phone(cap).unwrap_or(cap.get_match().as_str().to_string())
        });
        let s = self.re_address.replace_all(&s, |cap: &Captures| {
            try_address(cap).unwrap_or(cap.get_match().as_str().to_string())
        });
        let s = self.re_number.replace_all(&s, |cap: &Captures| {
            try_number(cap).unwrap_or(cap.get_match().as_str().to_string())
        });
        s.to_string()
    }
}

pub struct Preprocess {
    re_alphabet: Regex,
    re_long_spaces: Regex,
    replaces: Vec<(Regex, &'static str)>,
    kv_canonicalized: HashMap<String, String>,
    number_yomi: NumberYomi,
    symbols: HashMap<String, String>,
}

impl Preprocess {
    pub fn new() -> anyhow::Result<Self> {
        let re_alphabet = Regex::new("([a-zA-Z ]{5,}|[a-zA-Z]{1,5})")?;
        let re_long_spaces = Regex::new("[ \n\t]{2,}")?;
        let replaces = vec![
            (
                Regex::new(
                    r"[［＼］｛｝｟｠》「」『』【】〔〕〖〗〘〙〚〛▼♀♂《≪≫①②③④⑤⑥\u02d7\u2010-\u2015\u2043\u2212\u23af\u23e4\u2500\u2501\u2e3a\u2e3b]",
                )?,
                "",
            ),
            (Regex::new(r"[\uff5e\u301C]")?, "ー"),
            (Regex::new(r"[….]{3,}")?, "…"),
        ];
        let mut kv_canonicalized = HashMap::new();
        for line in all_csv::load_csv().lines() {
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() != 2 {
                anyhow::bail!("invalid csv")
            }
            kv_canonicalized.insert(cols[0].to_string(), cols[1].to_string());
        }
        let number_yomi = NumberYomi::new()?;
        let mut symbols = HashMap::new();
        for line in include_str!("../../../data/symbol.csv").lines() {
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() != 2 {
                anyhow::bail!("invalid csv")
            }
            symbols.insert(cols[0].to_string(), cols[1].to_string());
        }
        Ok(Self {
            re_alphabet,
            re_long_spaces,
            replaces,
            kv_canonicalized,
            number_yomi,
            symbols,
        })
    }
    pub fn run(&self, r: String) -> String {
        let r = r.nfkc().to_string();
        let mut r = self.re_long_spaces.replace_all(&r, " ").to_string();
        for (k, v) in &self.replaces {
            r = k.replace_all(&r, *v).to_string();
        }
        let r = self.re_alphabet.replace_all(&r, |cap: &Captures| {
            if let Some(r) = self.kv_canonicalized.get(
                &cap[0]
                    .to_lowercase()
                    .chars()
                    .filter(|c| c != &' ')
                    .collect::<String>(),
            ) {
                r.to_string()
            } else {
                cap[0]
                    .split(" ")
                    .filter(|c| !c.is_empty())
                    .map(|c| {
                        if let Some(c) = self.kv_canonicalized.get(&c.to_lowercase()) {
                            c
                        } else {
                            c
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        });
        let mut r = self.number_yomi.to_yomi(&r);
        for (k, v) in &self.symbols {
            r = r.split(k).collect::<Vec<_>>().join(v);
        }
        r.to_string()
    }
}

#[derive(Clone)]
pub struct SpliceText {
    re: Regex,
}

impl SpliceText {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            re: Regex::new(r"[。\n]+")?,
        })
    }
    pub fn splice(&self, text: &str) -> Vec<String> {
        self.re
            .split(text)
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(format!("{}。", s))
                }
            })
            .collect()
    }
}
