//! BTC Price Service with Moving Average
//!
//! Provides BTC price with rolling average to smooth volatility
//! Prevents contributors from being penalized by sudden price movements

use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use tracing::{info, warn};

/// BTC price with timestamp
#[derive(Debug, Clone)]
pub struct BtcPrice {
    pub price_usd: f64,
    pub timestamp: DateTime<Utc>,
}

/// BTC Price Service with Moving Average
pub struct BtcPriceService {
    /// Historical prices (timestamp, price_usd)
    prices: VecDeque<BtcPrice>,
    /// Moving average window in days (default: 30)
    ma_window_days: u32,
    /// Maximum number of price points to keep
    max_price_points: usize,
}

impl BtcPriceService {
    /// Create a new BTC price service
    pub fn new(ma_window_days: u32) -> Self {
        Self {
            prices: VecDeque::new(),
            ma_window_days,
            max_price_points: 1000, // Keep up to 1000 price points
        }
    }

    /// Add a new price point
    pub fn add_price(&mut self, price_usd: f64, timestamp: DateTime<Utc>) {
        // Add new price
        self.prices.push_back(BtcPrice {
            price_usd,
            timestamp,
        });

        // Trim old prices beyond window
        let cutoff = Utc::now() - chrono::Duration::days(self.ma_window_days as i64 + 7); // Keep 7 extra days
        while let Some(front) = self.prices.front() {
            if front.timestamp < cutoff {
                self.prices.pop_front();
            } else {
                break;
            }
        }

        // Trim if too many points
        while self.prices.len() > self.max_price_points {
            self.prices.pop_front();
        }
    }

    /// Get current moving average price
    pub fn get_moving_average(&self) -> f64 {
        let cutoff = Utc::now() - chrono::Duration::days(self.ma_window_days as i64);
        
        let recent_prices: Vec<f64> = self
            .prices
            .iter()
            .filter(|p| p.timestamp >= cutoff)
            .map(|p| p.price_usd)
            .collect();

        if recent_prices.is_empty() {
            warn!(
                "No price data in {} day window, using latest price or default",
                self.ma_window_days
            );
            // Fallback to latest price if available
            return self
                .prices
                .back()
                .map(|p| p.price_usd)
                .unwrap_or(50000.0); // Default fallback
        }

        let sum: f64 = recent_prices.iter().sum();
        let avg = sum / recent_prices.len() as f64;

        info!(
            "BTC price MA ({} days): ${:.2} (from {} price points)",
            self.ma_window_days, avg, recent_prices.len()
        );

        avg
    }

    /// Get latest price (not averaged)
    pub fn get_latest_price(&self) -> Option<f64> {
        self.prices.back().map(|p| p.price_usd)
    }

    /// Convert USD to BTC using moving average price
    pub fn usd_to_btc(&self, usd_amount: f64) -> f64 {
        let ma_price = self.get_moving_average();
        if ma_price <= 0.0 {
            warn!("Invalid BTC price, using default conversion");
            return usd_amount / 50000.0; // Fallback
        }
        usd_amount / ma_price
    }

    /// Get number of price points in window
    pub fn price_point_count(&self) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(self.ma_window_days as i64);
        self.prices
            .iter()
            .filter(|p| p.timestamp >= cutoff)
            .count()
    }
}

impl Default for BtcPriceService {
    fn default() -> Self {
        Self::new(30) // Default 30-day moving average
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average() {
        let mut service = BtcPriceService::new(30);

        // Add prices over 30 days
        let base_time = Utc::now() - chrono::Duration::days(30);
        for i in 0..30 {
            let price = 50000.0 + (i as f64 * 100.0); // Increasing prices
            let timestamp = base_time + chrono::Duration::days(i);
            service.add_price(price, timestamp);
        }

        // Moving average should be around middle of range
        let ma = service.get_moving_average();
        assert!(ma > 50000.0 && ma < 53000.0);
    }

    #[test]
    fn test_usd_to_btc_conversion() {
        let mut service = BtcPriceService::new(30);
        
        // Add some prices
        for i in 0..10 {
            service.add_price(50000.0, Utc::now() - chrono::Duration::days(i));
        }

        // $50,000 should convert to 1.0 BTC
        let btc = service.usd_to_btc(50000.0);
        assert!((btc - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_price_data() {
        let service = BtcPriceService::new(30);
        
        // Should return default when no data
        let ma = service.get_moving_average();
        assert_eq!(ma, 50000.0); // Default fallback
    }
}

