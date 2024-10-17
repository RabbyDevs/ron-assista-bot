use std::{collections::HashMap, fs, io::Write, path::Path, time::Duration};
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use serenity::all::{ChannelId, Context, Message};
use sled::Db;

use super::CONFIG;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct PolicyEntry {
    pub content: String,
    pub order: u64,
}

pub struct PolicySystem {
    db: Arc<Db>,
}

impl PolicySystem {
    pub fn init(db_path: &str) -> sled::Result<Self> {
        let db = Arc::new(sled::open(db_path)?);
        let system = PolicySystem {
            db: Arc::clone(&db)
        };

        Ok(system)
    }

    pub fn edit(&self, internal_name: &str, content: String, order: u64) -> sled::Result<()> {
        let entry = PolicyEntry { content, order };
        let serialized = bincode::serialize(&entry).map_err(|_| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Serialization error")))?;
        self.db.insert(internal_name, serialized)?;
        Ok(())
    }

    pub fn remove(&self, internal_name: &str) -> sled::Result<()> {
        self.db.remove(internal_name)?;
        Ok(())
    }

    pub fn list_policies(&self) -> sled::Result<Vec<(String, PolicyEntry)>> {
        let mut policies = Vec::new();
        
        for result in self.db.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8(key.to_vec()).map_err(|_| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "UTF-8 Error")))?;
            let entry: PolicyEntry = bincode::deserialize(&value)
                .map_err(|_| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Deserialization error")))?;
            
            policies.push((key_str, entry));
        }
        
        policies.sort_by_key(|(_, entry)| entry.order);

        Ok(policies)
    }

    pub fn list_policies_internal_names(&self) -> sled::Result<Vec<(String, PolicyEntry)>> {
        let mut policies = Vec::new();
        
        for result in self.db.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8(key.to_vec()).map_err(|_| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "UTF-8 Error")))?;
            let entry: PolicyEntry = bincode::deserialize(&value)
                .map_err(|_| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Deserialization error")))?;
            
            policies.push((key_str, entry));
        }

        policies.sort_by_key(|(_, entry)| entry.order);

        Ok(policies)
    }

    /// Updates the policies and sends messages to the relevant Discord channels
    pub async fn update_policy(&self, ctx: &Context) -> sled::Result<()> {
        let policies = self.list_policies()?;
    
        // Sort the policies by order and prepare the file contents
        let mut file_contents = String::new();
        for (_, policy) in policies.iter() {
            file_contents.push_str(&format!(
                "{}\n** **\n",
                policy.content
            ));
        }
    
        // Define paths for the policy files
        let previous_file_path = Path::new("policy.txt");
        let current_file_path = Path::new("current_policy.txt");
    
        // Compare with previous file if it exists
        if previous_file_path.exists() {
            let previous_content = fs::read_to_string(previous_file_path).unwrap_or_default();
            if previous_content != file_contents {
                // Send policy changes to the changes channel
                let changes_channel_id = CONFIG.modules.policy.policy_changes_channel_id.parse::<u64>().unwrap();
                let changes_channel = ctx.http.get_channel(changes_channel_id.into()).await.unwrap();
    
                let diff = diff_policies(&previous_content, &file_contents);
                send_long_message(ctx, &changes_channel.id(), &format!("Policy updates detected:\n```diff\n{}\n```", diff)).await;
            }
        }
    
        // Write the current policy to the file
        let mut file = fs::File::create(current_file_path)?;
        file.write_all(file_contents.as_bytes())?;
    
        // Send the current policy to the policy channel in sections
        let policy_channel_id = CONFIG.modules.policy.policy_channel_id.parse::<u64>().unwrap();
        let policy_channel = ctx.http.get_channel(policy_channel_id.into()).await.unwrap();
    
        let policy_actual_id = ChannelId::new(policy_channel_id);
        // Step 1: Delete all messages in the policy channel
        let mut message_stream = policy_actual_id.messages_iter(ctx).boxed();
        let mut messages_to_delete = Vec::new();
                        
        // Collect all message IDs
        while let Some(message_result) = message_stream.next().await {
            let message = message_result.unwrap();
            messages_to_delete.push(message.id);
        }
                        
        // Bulk delete messages in chunks of 100 (Discord's limit per request)
        while !messages_to_delete.is_empty() {
            let to_delete = messages_to_delete.split_off(messages_to_delete.len().saturating_sub(100));
            policy_actual_id.delete_messages(ctx, to_delete).await.unwrap();
            tokio::time::sleep(Duration::from_millis(1000)).await; // Avoid rate limits
        }
    
        let mut message_links = Vec::new();

        for (_, policy) in policies.iter() {
            let messages = send_long_message(ctx, &policy_channel.id(), &format!("{}\n** **", policy.content)).await;
            let first_heading = extract_first_heading(&policy.content);
            if let Some(heading) = first_heading {
                message_links.push((heading, messages.last().unwrap().link()));
            }
        }
    
        // Sort message_links by the numeric prefix of the heading
        message_links.sort_by(|(a, _), (b, _)| {
            let a_num = extract_number(a);
            let b_num = extract_number(b);
            a_num.cmp(&b_num)
        });
    
        let mut toc_content = String::new();
    
        // Prepare the table of contents with links
        for (index, (heading, link)) in message_links.iter().enumerate() {
            toc_content.push_str(&format!("{}. [{}]({})\n", index + 1, heading, link));
        }
    
        // Send the table of contents, splitting if necessary
        send_long_message(ctx, &policy_channel.id(), &format!("# Table of Contents:\n{}", toc_content)).await;
    
        // Move current policy file to previous
        fs::rename(current_file_path, previous_file_path)?;
    
        Ok(())
    }

    pub fn clear_all(&self) -> sled::Result<()> {
        let policies = self.list_policies_internal_names()?;
        for (internal_name, _) in policies {
            self.db.remove(internal_name)?;
        }
        Ok(())
    }
}

// Helper function to diff policies and return the changes
fn diff_policies(previous: &str, current: &str) -> String {
    use similar::{TextDiff, ChangeTag};

    let mut changes = String::new();
    let diff = TextDiff::from_lines(previous, current);

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => changes.push_str(&format!("- {}", change)),
            ChangeTag::Insert => changes.push_str(&format!("+ {}", change)),
            _ => {}
        }
    }

    changes
}

fn remove_hash_from_first_line(input: &str) -> String {
    // Get the first line of the input string
    let first_line = input.lines().next().unwrap_or("");
    
    // Remove '#' characters from the beginning of the line
    let trimmed_line = first_line.trim_start_matches('#');
    
    trimmed_line.to_string()
}

    // Helper function to send long messages
async fn send_long_message(ctx: &Context, channel_id: &ChannelId, content: &str) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        let (chunk, rest) = split_message(remaining);
        let message = channel_id.say(ctx, chunk).await.unwrap();
        messages.push(message);
        remaining = rest;
    }

    messages
}

// Helper function to split a message into chunks
fn split_message(content: &str) -> (&str, &str) {
    if content.len() <= 2000 {
        return (content, "");
    }

    let mut split_index = 2000;
    while !content.is_char_boundary(split_index) {
        split_index -= 1;
    }

    content.split_at(split_index)
}

// Helper function to extract the first heading from the content
fn extract_first_heading(content: &str) -> Option<String> {
    content.lines()
        .find(|line| line.starts_with('#'))
        .map(|line| line.trim_start_matches('#').trim().to_string())
}

// Helper function to extract the numeric prefix from a heading
fn extract_number(heading: &str) -> u32 {
    heading.split_whitespace()
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(u32::MAX)  // If parsing fails, consider it as the largest number
}
