use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use sled::Db;
use futures::future::BoxFuture;

pub struct UserTimer {
    end_time: Instant,
    role_id: String,
    paused_at: Option<Instant>,
    paused_duration: Duration,
}

pub struct TimerSystem {
    db: Arc<Db>,
    timers: Arc<Mutex<HashMap<String, UserTimer>>>,
    event_handler: Arc<Mutex<Box<dyn Fn(String, String) -> BoxFuture<'static, ()> + Send + Sync>>>,
}

impl TimerSystem {
    pub fn new(db_path: &str) -> sled::Result<Self> {
        let db = Arc::new(sled::open(db_path)?);
        let timers = Arc::new(Mutex::new(HashMap::new()));
        let event_handler: Arc<Mutex<Box<dyn Fn(String, String) -> BoxFuture<'static, ()> + Send + Sync>>> = 
            Arc::new(Mutex::new(Box::new(|_: String, _: String| Box::pin(async {}))));

        let system = TimerSystem {
            db: Arc::clone(&db),
            timers: Arc::clone(&timers),
            event_handler,
        };

        // Load existing timers from the database
        for result in db.iter() {
            let (key, value) = result?;
            let user_id = String::from_utf8(key.to_vec()).unwrap();
            let (end_timestamp, role_id, is_paused, paused_duration) = TimerSystem::deserialize_db_value(&value);
            let end_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(end_timestamp);
            if end_time > std::time::SystemTime::now() {
                let duration = end_time.duration_since(std::time::SystemTime::now()).unwrap();
                system.add_timer(user_id, role_id, duration.as_secs(), is_paused, paused_duration)?;
            } else {
                db.remove(key)?;
            }
        }

        Ok(system)
    }

    pub fn add_timer(&self, user_id: String, role_id: String, duration_secs: u64, is_paused: bool, paused_duration: Option<u64>) -> sled::Result<()> {
        let end_time = Instant::now() + Duration::from_secs(duration_secs);
        let end_timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() + duration_secs;

        let timers = self.timers.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let mut timers = timers.lock().await;
            let timer = UserTimer {
                end_time,
                role_id: role_id.clone(),
                paused_at: if is_paused { Some(Instant::now()) } else { None },
                paused_duration: paused_duration.map(Duration::from_secs).unwrap_or(Duration::from_secs(0)),
            };
            timers.insert(user_id.clone(), timer);
            let db_value = Self::serialize_db_value(end_timestamp, &role_id, is_paused, paused_duration);
            db.insert(user_id, db_value).unwrap();
        });

        Ok(())
    }

    pub async fn pause_timer(&self, user_id: &str) -> Result<(), String> {
        let mut timers = self.timers.lock().await;
        if let Some(timer) = timers.get_mut(user_id) {
            if timer.paused_at.is_none() {
                let now = Instant::now();
                timer.paused_at = Some(now);
                
                let (end_timestamp, role_id, _, _) = TimerSystem::deserialize_db_value(&self.db.get(user_id).unwrap().unwrap());
                let db_value = Self::serialize_db_value(end_timestamp, &role_id, true, Some(timer.paused_duration.as_secs()));
                self.db.insert(user_id, db_value).unwrap();
                
                Ok(())
            } else {
                Err("Timer is already paused".to_string())
            }
        } else {
            Err("Timer not found".to_string())
        }
    }

    pub async fn resume_timer(&self, user_id: &str) -> Result<String, String> {
        let mut timers = self.timers.lock().await;
        if let Some(timer) = timers.get_mut(user_id) {
            if let Some(paused_at) = timer.paused_at {
                let now = Instant::now();
                let additional_pause = now.duration_since(paused_at);
                timer.paused_duration += additional_pause;
                timer.end_time += additional_pause;
                timer.paused_at = None;
                
                let new_end_timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() + timer.end_time.duration_since(now).as_secs();
                let db_value = Self::serialize_db_value(new_end_timestamp, &timer.role_id, false, Some(timer.paused_duration.as_secs()));
                self.db.insert(user_id, db_value).unwrap();
                
                Ok(timer.role_id.clone())
            } else {
                Err("Timer is not paused".to_string())
            }
        } else {
            Err("Timer not found".to_string())
        }
    }

    fn serialize_db_value(timestamp: u64, role_id: &str, is_paused: bool, paused_duration: Option<u64>) -> Vec<u8> {
        let mut value = timestamp.to_be_bytes().to_vec();
        value.extend_from_slice(role_id.as_bytes());
        value.push(if is_paused { 1 } else { 0 });
        if let Some(duration) = paused_duration {
            value.extend_from_slice(&duration.to_be_bytes());
        }
        value
    }

    fn deserialize_db_value(value: &[u8]) -> (u64, String, bool, Option<u64>) {
        let timestamp = u64::from_be_bytes(value[..8].try_into().unwrap());
        let role_id_end = value.iter().skip(8).position(|&x| x == 0 || x == 1).unwrap() + 8;
        let role_id = String::from_utf8(value[8..role_id_end].to_vec()).unwrap();
        let is_paused = value[role_id_end] == 1;
        let paused_duration = if value.len() > role_id_end + 1 {
            Some(u64::from_be_bytes(value[role_id_end+1..].try_into().unwrap()))
        } else {
            None
        };
        (timestamp, role_id, is_paused, paused_duration)
    }

    pub fn start_timer_thread(&self) {
        let timers = Arc::clone(&self.timers);
        let db = Arc::clone(&self.db);
        let event_handler = Arc::clone(&self.event_handler);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let mut timers = timers.lock().await;
                let now = Instant::now();
                let system_now = std::time::SystemTime::now();
                
                let expired_timers: Vec<(String, String)> = timers
                    .iter()
                    .filter(|(_, timer)| timer.paused_at.is_none() && timer.end_time <= now)
                    .map(|(user_id, timer)| (user_id.clone(), timer.role_id.clone()))
                    .collect();

                for (user_id, role_id) in &expired_timers {
                    db.remove(user_id).unwrap();
                    let handler = event_handler.lock().await;
                    handler(user_id.clone(), role_id.clone()).await;
                }

                // Remove expired timers
                for (user_id, _) in expired_timers {
                    timers.remove(&user_id);
                }

                // Update remaining timers in the database
                for (user_id, timer) in timers.iter() {
                    if timer.paused_at.is_none() && timer.end_time > now {
                        let remaining = timer.end_time - now;
                        let end_time = system_now + std::time::Duration::from_secs(remaining.as_secs());
                        let end_timestamp = end_time.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
                        let db_value = Self::serialize_db_value(end_timestamp, &timer.role_id, false, Some(timer.paused_duration.as_secs()));
                        db.insert(user_id, db_value).unwrap();
                    }
                }
            }
        });
    }

    pub async fn set_event_handler<F, Fut>(&self, handler: F)
    where
        F: Fn(String, String) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = ()> + Send + 'static,
    {
        *self.event_handler.lock().await = Box::new(move |user_id, role_id| Box::pin(handler(user_id, role_id)));
    }

    pub async fn delete_timer(&self, user_id: &str) -> Result<(), String> {
        let mut timers = self.timers.lock().await;
        if timers.remove(user_id).is_some() {
            // Timer was found and removed from in-memory HashMap
            match self.db.remove(user_id) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to remove timer from database: {}", e)),
            }
        } else {
            Err("Timer not found".to_string())
        }
    }
}