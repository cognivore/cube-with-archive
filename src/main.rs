use reqwest;
use std::collections::HashMap;

fn download_card_list(url: &str) -> Result<String, reqwest::Error> {
    // Reqwest blocking get
    reqwest::blocking::get(url)?.text()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    Special,
}
impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Rarity::Common => write!(f, "Common"),
            Rarity::Uncommon => write!(f, "Uncommon"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Mythic => write!(f, "Mythic"),
            Rarity::Special => write!(f, "Special"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SlotValue {
    rarity: Rarity,
    count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Layout {
    weight: i32,
    slots: Vec<SlotValue>, // It's cringe how we overload semantics of count here, but ok.
}
type Name = String;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Layouts {
    value: HashMap<Name, Layout>,
}
impl std::fmt::Display for Layouts {
    // Print in the following format:
    /*
    {
        "layouts": {
            "Rare": {
                "weight": 7,
                "slots": {
                      "Common": 10,
                      "Uncommon": 3,
                      "Rare": 1
                }
            },
            "Mythic": {
                "weight": 1,
                "slots": {
                      "Common": 10,
                      "Uncommon": 3,
                      "Mythic": 1
                }
            }
        }
    }
    */
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{\n  \"layouts\": {{\n")?;
        for (name, layout) in &self.value {
            write!(f, "    \"{}\": {{\n", name)?;
            write!(f, "      \"weight\": {},\n", layout.weight)?;
            write!(f, "      \"slots\": {{\n")?;
            for slot in &layout.slots {
                write!(f, "        \"{}\": {},\n", slot.rarity, slot.count)?;
            }
            write!(f, "      }}\n")?;
            write!(f, "    }}\n")?;
        }
        write!(f, "  }}\n")?;
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Slot {
    values: Vec<SlotValue>,
}

fn s(rarity: Rarity) -> Slot {
    Slot {
        values: vec![SlotValue { rarity, count: 1 }],
    }
}

fn rare() -> Slot {
    Slot {
        values: vec![
            SlotValue {
                rarity: Rarity::Rare,
                count: 7,
            },
            SlotValue {
                rarity: Rarity::Mythic,
                count: 1,
            },
        ],
    }
}

fn get_layout(cube_id: &str) -> HashMap<Slot, i32> {
    return match cube_id {
        "garbagemasters" => {
            let mut res = HashMap::new();
            res.insert(s(Rarity::Common), 11);
            res.insert(s(Rarity::Uncommon), 4);
            res.insert(rare(), 1);
            res.insert(s(Rarity::Special), 2);
            res
        }
        _ => {
            let mut res = HashMap::new();
            res.insert(s(Rarity::Common), 11);
            res.insert(s(Rarity::Uncommon), 3);
            res.insert(rare(), 1);
            res
        }
    };
}

fn unfurl_layout(raw_layout: HashMap<Slot, i32>) -> Layouts {
    // Memorize each slot that just has one element
    let mut singletons = HashMap::new();

    for (slot, count) in &raw_layout {
        if slot.values.len() == 1 {
            singletons.insert(slot.values[0].rarity.clone(), count);
        }
    }

    // Now check that there is not more than one slot with more than one element, and if there
    // isn't, create the amount of layouts equal to the amount of slot values in that slot, adding
    // all the singletons in.

    if raw_layout
        .iter()
        .filter(|(slot, _)| slot.values.len() > 1)
        .count()
        > 1
    {
        panic!("Only one slot can have more than one value");
    }

    let mut layouts = Layouts {
        value: HashMap::new(),
    };
    for (slot, count) in &raw_layout {
        if slot.values.len() == 1 {
            continue;
        }
        for value in &slot.values {
            let mut layout = Layout {
                weight: value.count,
                slots: Vec::new(),
            };
            layout.slots.push(SlotValue {
                rarity: value.rarity.clone(),
                count: *count,
            });
            for (rarity, count) in &singletons {
                layout.slots.push(SlotValue {
                    rarity: rarity.clone(),
                    count: **count,
                });
            }
            layouts.value.insert(value.rarity.to_string(), layout);
        }
    }

    return layouts;
}

fn parse_csv_card_list(data: &str) -> HashMap<Rarity, Vec<String>> {
    /*
    name,CMC,Type,Color,Set,Collector Number,Rarity,Color Category,status,Finish,maybeboard,image URL,image Back URL,tags,Notes,MTGO ID
    "Humility",4,"Enchantment",W,"tpr","16",mythic,w,Owned,Non-foil,false,,,"","",56658
    "Archangel's Light",8,"Sorcery",W,"dka","1",mythic,w,Owned,Non-foil,false,,,"","",43215
    "Amrou Kithkin",2,"Creature - Kithkin",W,"me3","3",special,w,Owned,Non-foil,false,,,"","",33482
    */
    let mut res = HashMap::new();
    for line in data.lines().skip(1) {
        // Parse CSV respecting quotation marks in plain Rust, without using libraries
        let mut parts = Vec::new();
        let mut in_quotes = false;
        let mut current = String::new();
        for c in line.chars() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ',' if !in_quotes => {
                    parts.push(current);
                    current = String::new();
                }
                _ => {
                    current.push(c);
                }
            }
        }
        parts.push(current);

        // Debug collected version of parts
        dbg!(&parts);

        let mut psi = parts.iter();

        let name = psi.nth(0).unwrap();
        let rarity = psi.nth(5).unwrap();
        let rarity = match rarity.as_str() {
            "common" => Rarity::Common,
            "uncommon" => Rarity::Uncommon,
            "rare" => Rarity::Rare,
            "mythic" => Rarity::Mythic,
            "special" => Rarity::Special,
            _ => {
                dbg!(name, rarity);
                panic!("Unknown rarity")
            }
        };
        let entry = res.entry(rarity).or_insert(Vec::new());
        entry.push(name.to_string());
    }
    res
}

fn csv_card_list_to_draftmancer(xkv: HashMap<Rarity, Vec<String>>) -> String {
    /*
    [Common]
    ..Card List..
    [Uncommon]
    ..Card List..
    [Rare]
    ..Card List..
    [Mythic]
    ..Card List..
    */
    let mut res = String::new();
    for (rarity, cards) in xkv {
        res.push_str(&format!("[{}]\n", rarity));
        for card in cards {
            res.push_str(&format!("{}\n", card));
        }
    }
    res
}

fn settings_and_data_to_draftmancer(layouts: &Layouts, data: &str) -> String {
    let mut res = String::new();
    res.push_str("[Settings]\n");
    res.push_str(format!("{}\n", layouts).as_str());
    res.push_str(csv_card_list_to_draftmancer(parse_csv_card_list(data)).as_str());
    res
}

fn parse_card_list(data: &str) -> (Vec<String>, Vec<String>) {
    let mut in_mainboard = true;
    let mut cards = HashMap::new();
    let mut duplicates = Vec::new();

    for line in data.lines() {
        let line = line.trim();
        if line.starts_with("# mainboard") {
            in_mainboard = true;
        } else if line.starts_with("#") {
            in_mainboard = false;
        } else if in_mainboard && !line.is_empty() {
            let count = cards.entry(line.to_string()).or_insert(0);
            *count += 1;
            if *count > 1 {
                duplicates.push(line.to_string());
            }
        }
    }

    let unique_cards = cards.keys().cloned().collect::<Vec<_>>();
    (unique_cards, duplicates)
}

fn generate_draftmancer_list(cards: &[String], duplicates: &[String]) -> String {
    let mut result = String::from("[Layouts]\n- Archive (1)\n\t14 Cubed\n\t1 Archived\n[Cubed]\n");
    for card in cards {
        result.push_str(card);
        result.push('\n');
    }
    result.push_str("[Archived]\n");
    for card in duplicates {
        result.push_str(card);
        result.push('\n');
    }
    result
}

fn main() {
    // Check if we are getting Into the Story cube, if so, do the thing.
    let arg_cube_type = std::env::args().nth(1);
    if arg_cube_type == Some("its".to_string()) || arg_cube_type == Some("IntoTheStory".to_string())
    {
        return into_the_story();
    }
    if arg_cube_type == Some("rema".to_string())
        || arg_cube_type == Some("RemasteringMagic".to_string())
    {
        if let Some(arg_cube_id) = std::env::args().nth(2) {
            return remastering_magic(arg_cube_id);
        } else {
            return eprintln!("Please provide a cube ID for Remastering Magic");
        }
    }
}

fn remastering_magic(cube_id: String) {
    let cubecobra_csv_url = format!(
        "https://cubecobra.com/cube/download/csv/{}?primary=Color%20Category&secondary=Rarity&tertiary=Creature%2FNon-Creature&quaternary=Mana%20Value&showother=false",
        cube_id
    );
    /*
     * name,CMC,Type,Color,Set,Collector Number,Rarity,Color Category,status,Finish,maybeboard,image URL,image Back URL,tags,Notes,MTGO ID
     * "Icatian Moneychanger",1,"Creature - Human",W,"fem","10c",common,w,Owned,Non-foil,false,,,"","",
     */
    let data = download_card_list(&cubecobra_csv_url).unwrap();
    let layouts = unfurl_layout(get_layout(&cube_id));
    let draftmancer_list = settings_and_data_to_draftmancer(&layouts, &data);
    let filename = format!("{}.txt", cube_id);
    std::fs::write(&filename, &draftmancer_list).unwrap();
    println!("{}", &draftmancer_list);
}

fn into_the_story() {
    let url = "https://cubecobra.com/cube/download/plaintext/633f463453859b175ba27b36?primary=Color%20Category&secondary=Types-Multicolor&tertiary=Mana%20Value&quaternary=Alphabetical&showother=undefined";
    match download_card_list(url) {
        Ok(data) => {
            let (cards, duplicates) = parse_card_list(&data);
            let draftmancer_list = generate_draftmancer_list(&cards, &duplicates);
            println!("{}", draftmancer_list);
        }
        Err(e) => {
            eprintln!("Error downloading card list: {}", e);
        }
    }
}
