use reqwest;
use std::collections::HashMap;

fn download_card_list(url: &str) -> Result<String, reqwest::Error> {
    // Reqwest blocking get
    reqwest::blocking::get(url)?.text()
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
    result.push_str("[Archive]\n");
    for card in duplicates {
        result.push_str(card);
        result.push('\n');
    }
    result
}

fn main() {
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
