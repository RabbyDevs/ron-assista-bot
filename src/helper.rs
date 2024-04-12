use indexmap::IndexMap;
use regex::Regex;
use reqwest::header::HeaderValue;
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{UserId, CONFIG, REQWEST_CLIENT};

pub async fn discord_id_to_roblox_id(discord_id: UserId) -> Result<String, String> {
    let quote_regex = Regex::new("/\"/gi").expect("regex err");
    let bloxlink_api_key: HeaderValue = CONFIG.bloxlink_api_key.parse::<HeaderValue>().expect("err");

    let url = format!("https://api.blox.link/v4/public/discord-to-roblox/{}", discord_id.to_string());
    let response = REQWEST_CLIENT.get(url)
        .header("Authorization", bloxlink_api_key)
        .send()
        .await.expect("??");
    if response.status() != reqwest::StatusCode::OK {
        Err(format!("Something went wrong attempting to get Bloxlink data for user `{}`. They might not be verified with Bloxlink.", discord_id))
    } else {
        let serialized_json: Value = serde_json::from_str(response.text().await.expect("err").as_str()).expect("err");
        Ok(quote_regex.replace(serialized_json["robloxID"].as_str().unwrap(), "").to_string())
    }
}

pub async fn duration_conversion(duration_string: String) -> Result<(u64, u64, String), String> {
    let mut date_map = HashMap::new();
    date_map.insert("h", (3600, "Hour"));
    date_map.insert("d", (86400, "Day"));
    date_map.insert("w", (604800, "Week"));
    date_map.insert("m", (2629743, "Month"));
    let duration_list = duration_string.split(" ").map(str::to_string).collect::<Vec<String>>();
    let mut unix_total = 0;
    let mut final_string = String::new();
    if duration_list.len() == 0 {return Err(format!("Something went wrong parsing duration string `{}`.", duration_string));} else {
        for duration in duration_list.clone() {
            let chars = duration.chars();
            let amount = match chars.clone().filter(|x| x.is_ascii_digit()).collect::<String>().parse::<u64>() {
                Ok(amount) => amount,
                Err(_) => {
                    return Err(format!("Something went wrong parsing duration string `{}`.", duration_string));
                },
            };
            let identifier = chars.last().expect("err");
            if date_map.contains_key(identifier.to_string().as_str()) == false {
                return Err(format!("Something went wrong parsing duration string `{}`.", duration_string));
            }
            let mut name = date_map[&identifier.to_string().as_str()].1.to_string();
            if amount > 1 {name = format!("{} {}s, ", amount, name)} else {name = format!("{} {}, ", amount, name)}
            if duration_list.ends_with(&[duration.clone()]) == true {name.pop();name.pop();}
            if duration_list.ends_with(&[duration.clone()]) == true && duration_list.starts_with(&[duration.clone()]) != true {name = format!("and {}", name);}
            final_string.push_str(name.as_str());
            let unix_unit = date_map[&identifier.to_string().as_str()].0 * amount;
            unix_total += unix_unit
        }
    }
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let epoch_in_s = since_the_epoch.as_secs();
    Ok((epoch_in_s, epoch_in_s + unix_total, final_string))
}

pub async fn badge_data(roblox_id: String, badge_iterations: i64) -> Result<(i64, i64, i64, IndexMap<u64, u64>), String> {
    let regex = Regex::new("/Welcome|Join|visit|play/gi").expect("regex err");
    let quote_regex = Regex::new("/\"/gi").expect("regex err");

    let mut badge_count: i64 = 0;
	let mut total_win_rate: i64 = 0;
	let mut win_rate : i64= 0;
	let mut welcome_badge_count: i64 = 0;
	let mut cursor: String = String::new();
	let mut awarders: IndexMap<u64, u64> = IndexMap::new();
    for _ in 0..badge_iterations {
        if cursor == "null" {break}
        let url = if cursor.len() != 0 { format!("https://badges.roblox.com/v1/users/{}/badges?limit=100&sortOrder=Asc&cursor={}", roblox_id, cursor) } 
        else {format!("https://badges.roblox.com/v1/users/{}/badges?limit=100&sortOrder=Asc", roblox_id)};
        let response = REQWEST_CLIENT.get(url)
            .send()
            .await.expect("??");
        if response.status() != reqwest::StatusCode::OK {
            return Err("Request failed.".to_string());
        } else {
            let parsed_json: Value = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
            badge_count += parsed_json["data"].as_array().unwrap().len() as i64;
            if badge_count == 0 {break}
            cursor = quote_regex.replace(parsed_json["nextPageCursor"].as_str().unwrap(), "").to_string();
            for badge_data in parsed_json["data"].as_array().unwrap() {
				total_win_rate += badge_data["statistics"]["winRatePercentage"].as_number().unwrap().as_f64().unwrap() as i64;
				if regex.is_match(badge_data["name"].as_str().unwrap()) == true {
					welcome_badge_count += 1
				}
                let awarder_index = badge_data["awarder"]["id"].as_number().unwrap().as_u64().unwrap();
                let awarder = awarders.get_mut(awarder_index.borrow());
                match awarder {
                    Some(awarder) => {*awarder += 1},
                    None => {awarders.insert(awarder_index, 1);},
                };
			}
        }
    }
    if badge_count > 0 {win_rate += (total_win_rate*100)/badge_count};
    awarders.sort_by(|_, b: &u64, _, d: &u64| d.cmp(&b));
    Ok((badge_count, win_rate, welcome_badge_count, awarders))
}

pub async fn roblox_friend_count(roblox_id: String) -> usize {
    let response = REQWEST_CLIENT.get(format!("https://friends.roblox.com/v1/users/{}/friends", roblox_id)).send().await.expect("request err");
    let parsed_json: Value = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    parsed_json["data"].as_array().unwrap().len()
}

pub async fn roblox_group_count(roblox_id: String) -> usize {
    let response = REQWEST_CLIENT.get(format!("https://groups.roblox.com/v2/users/{}/groups/roles?includeLocked=true", roblox_id)).send().await.expect("request err");
    let parsed_json: Value = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    parsed_json["data"].as_array().unwrap().len()
}