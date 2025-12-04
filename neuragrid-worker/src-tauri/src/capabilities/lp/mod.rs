use async_trait::async_trait;
use super::Capability;
use serde::Serialize;
use rand::Rng;
use chrono::{Local, Duration, TimeZone};

#[derive(Serialize)]
struct LpResult {
    vehicle_number: String,
    video_url: String,
    timestamps: Vec<String>,
}

pub struct LicensePlateRecognition;

#[async_trait]
impl Capability for LicensePlateRecognition {
    fn code(&self) -> &'static str {
        "Lp"
    }

    async fn is_supported(&self) -> bool {
        true
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, String> {
        if args.len() < 2 {
            return Err("Usage: <vehicle_number> <video_url>".to_string());
        }
        let vehicle_number = args[0].clone();
        let video_url = args[1].clone();

        // Simulate processing time: 2 to 3 minutes (120 to 180 seconds)
        let sleep_seconds = {
            let mut rng = rand::thread_rng();
            rng.gen_range(120..=180)
        };
        
        // We can log this if we had access to the app handle, but we don't here easily without changing the trait signature or passing it in.
        // For now, just sleep.
        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_seconds)).await;

        let mut rng = rand::thread_rng();
        let mut timestamps = Vec::new();
        
        // Base time: Current time
        let now = Local::now();

        for _ in 0..5 {
            // Generate random offset in seconds (up to 1 hour back)
            let offset_secs = rng.gen_range(0..3600);
            let ts = now - Duration::seconds(offset_secs);
            timestamps.push(ts.format("%d/%m/%y %H:%M:%S").to_string());
        }
        timestamps.sort(); // Sort chronologically

        let result = LpResult {
            vehicle_number,
            video_url,
            timestamps,
        };

        serde_json::to_string(&result).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lp_execution() {
        let lp = LicensePlateRecognition;
        let args = vec!["KA01AB1234".to_string(), "http://example.com/video.mp4".to_string()];
        let result = lp.execute(args).await;
        assert!(result.is_ok());
        
        let json = result.unwrap();
        println!("Lp Output: {}", json);
        
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["vehicle_number"], "KA01AB1234");
        assert_eq!(parsed["video_url"], "http://example.com/video.mp4");
        assert!(parsed["timestamps"].is_array());
        
        let timestamps = parsed["timestamps"].as_array().unwrap();
        assert_eq!(timestamps.len(), 5);
        
        // Verify format
        let ts_str = timestamps[0].as_str().unwrap();
        // Simple regex-like check or just parsing
        assert!(chrono::NaiveDateTime::parse_from_str(ts_str, "%d/%m/%y %H:%M:%S").is_ok());
    }
}
