use indexmap::IndexMap;
use regex::Regex;
use reqwest::header::HeaderValue;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use super::{UserId, CONFIG, REQWEST_CLIENT, RBX_CLIENT};

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
    let duration_list = duration_string.split(' ').map(str::to_string).collect::<Vec<String>>();
    let mut unix_total = 0;
    let mut final_string = String::new();
    if duration_list.is_empty() {return Err(format!("Something went wrong parsing duration string `{}`.", duration_string));} else {
        for duration in duration_list.clone() {
            let chars = duration.chars();
            let amount = match chars.clone().filter(|x| x.is_ascii_digit()).collect::<String>().parse::<u64>() {
                Ok(amount) => amount,
                Err(_) => {
                    return Err(format!("Something went wrong parsing duration string `{}`.", duration_string));
                },
            };
            let identifier = chars.last().expect("err");
            if !date_map.contains_key(identifier.to_string().as_str()) {
                return Err(format!("Something went wrong parsing duration string `{}`.", duration_string));
            }
            let mut name = date_map[&identifier.to_string().as_str()].1.to_string();
            if amount > 1 {name = format!("{} {}s, ", amount, name)} else {name = format!("{} {}, ", amount, name)}
            if duration_list.ends_with(&[duration.clone()]) {name.pop();name.pop();}
            if duration_list.ends_with(&[duration.clone()]) && !duration_list.starts_with(&[duration.clone()]) {name = format!("and {}", name);}
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

pub async fn badge_data(roblox_id: String, badge_iterations: i64) -> Result<(i64, f64, i64, String), String> {
    let regex = Regex::new(r"(Welcome|Join|visit|play)").expect("regex err");
    let quote_regex = Regex::new(r#"\""#).expect("regex err");

    let mut badge_count = 0;
    let mut total_win_rate = 0.0;
    let mut win_rate = 0.0;
    let mut welcome_badge_count = 0;
    let mut cursor = String::new();
    let mut awarders = IndexMap::new();

    for _ in 0..badge_iterations {
        if cursor == "null" {
            break;
        }

        let url = if cursor.is_empty() {
            format!("https://badges.roblox.com/v1/users/{}/badges?limit=100&sortOrder=Asc", roblox_id)
        } else {
            format!("https://badges.roblox.com/v1/users/{}/badges?limit=100&sortOrder=Asc&cursor={}", roblox_id, cursor)
        };

        let response = REQWEST_CLIENT.get(&url)
            .send()
            .await
            .unwrap_or_else(|e| panic!("Request failed: {}", e));

        if !response.status().is_success() {
            return Err("Request failed.".to_string());
        }

        let parsed_json: Value = serde_json::from_str(&response.text().await.unwrap_or_else(|e| panic!("Failed to parse response: {}", e)))
            .unwrap_or_else(|e| panic!("Failed to parse JSON: {}", e));

        let next_page_cursor = parsed_json.get("nextPageCursor").and_then(|c| c.as_str()).unwrap_or("null");
        let data = parsed_json.get("data").and_then(|d| d.as_array()).unwrap();
        badge_count += data.len() as i64;

        if badge_count != 0 && next_page_cursor != "null" {
            cursor = quote_regex.replace(next_page_cursor, "").to_string();
        } else {
            cursor = "null".to_string();
        }

        for badge_data in data {
            total_win_rate += badge_data["statistics"]["winRatePercentage"].as_f64().unwrap();
            if regex.is_match(badge_data["name"].as_str().unwrap()) {
                welcome_badge_count += 1;
            }
            let awarder_index = badge_data["awarder"]["id"].as_u64().unwrap();
            awarders.entry(awarder_index).and_modify(|e| *e += 1).or_insert(1);
        }
    }

    if badge_count > 0 {
        win_rate += (total_win_rate * 100.0) / badge_count as f64;
    }

    let mut awarders_vec: Vec<_> = awarders.into_iter().collect();
    awarders_vec.sort_by(|(_, a), (_, b)| b.cmp(a));
    awarders_vec.truncate(5);

    let awarders_string = if awarders_vec.is_empty() {
        "No badges found, there are no top badge givers.".to_string()
    } else {
        awarders_vec.iter().map(|(id, count)| format!("\n - {}: {}", id, count)).collect()
    };

    Ok((badge_count, win_rate, welcome_badge_count, awarders_string))
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

pub async fn merge_types(mut roblox_users: Vec<String>, discord_ids: Vec<String>, mut roblox_ids: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut errors_vector = Vec::new();
    if roblox_users[0].is_empty() && discord_ids[0].is_empty() && roblox_ids[0].is_empty() {
        errors_vector.push("Command failed; no users inputted, or users improperly inputted.".to_string())
    }

    if roblox_users[0].is_empty() {roblox_users.remove(0);}
    let user_search = RBX_CLIENT.username_user_details(roblox_users.clone(), false).await.unwrap();
    if user_search.len() != roblox_users.len() {errors_vector.push("One or more Roblox user(s) failed to process, likely failing to be found as a valid Roblox username, make sure you properly input user(s).".to_string());}
    for user in user_search {
        roblox_ids.push(user.id.to_string())
    }

    for id in discord_ids {
        if id.is_empty() {continue}
        let discord_id = UserId::from_str(id.as_str()).expect("err");
        let roblox_id_str = match self::discord_id_to_roblox_id(discord_id).await {Ok(id) => id, Err(err) => {
            errors_vector.push(err);
            continue

        }};
        roblox_ids.push(roblox_id_str);
    }
    (roblox_ids, errors_vector)
}