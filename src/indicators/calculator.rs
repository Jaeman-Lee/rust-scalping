use ta::indicators::{BollingerBands, ExponentialMovingAverage, RelativeStrengthIndex};
use ta::Next;

pub struct IndicatorCalculator {
    ema_short: ExponentialMovingAverage,
    ema_long: ExponentialMovingAverage,
    rsi: RelativeStrengthIndex,
    bollinger: BollingerBands,
    prev_ema_short: Option<f64>,
    prev_ema_long: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct IndicatorValues {
    pub ema_short: f64,
    pub ema_long: f64,
    pub prev_ema_short: Option<f64>,
    pub prev_ema_long: Option<f64>,
    pub rsi: f64,
    pub bb_upper: f64,
    pub bb_middle: f64,
    pub bb_lower: f64,
    pub price: f64,
}

impl IndicatorCalculator {
    pub fn new(
        ema_short_period: usize,
        ema_long_period: usize,
        rsi_period: usize,
        bb_period: usize,
        bb_std: f64,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            ema_short: ExponentialMovingAverage::new(ema_short_period)
                .map_err(|e| anyhow::anyhow!("EMA short init error: {}", e))?,
            ema_long: ExponentialMovingAverage::new(ema_long_period)
                .map_err(|e| anyhow::anyhow!("EMA long init error: {}", e))?,
            rsi: RelativeStrengthIndex::new(rsi_period)
                .map_err(|e| anyhow::anyhow!("RSI init error: {}", e))?,
            bollinger: BollingerBands::new(bb_period, bb_std)
                .map_err(|e| anyhow::anyhow!("Bollinger init error: {}", e))?,
            prev_ema_short: None,
            prev_ema_long: None,
        })
    }

    pub fn update(&mut self, price: f64) -> IndicatorValues {
        let prev_short = self.prev_ema_short;
        let prev_long = self.prev_ema_long;

        let ema_short_val = self.ema_short.next(price);
        let ema_long_val = self.ema_long.next(price);
        let rsi_val = self.rsi.next(price);
        let bb = self.bollinger.next(price);

        self.prev_ema_short = Some(ema_short_val);
        self.prev_ema_long = Some(ema_long_val);

        IndicatorValues {
            ema_short: ema_short_val,
            ema_long: ema_long_val,
            prev_ema_short: prev_short,
            prev_ema_long: prev_long,
            rsi: rsi_val,
            bb_upper: bb.upper,
            bb_middle: bb.average,
            bb_lower: bb.lower,
            price,
        }
    }

    /// Feed historical data to warm up the indicators
    pub fn warm_up(&mut self, prices: &[f64]) {
        for &price in prices {
            self.update(price);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_calculator() -> IndicatorCalculator {
        IndicatorCalculator::new(9, 21, 14, 20, 2.0).unwrap()
    }

    #[test]
    fn test_new_valid_params() {
        let calc = IndicatorCalculator::new(9, 21, 14, 20, 2.0);
        assert!(calc.is_ok());
    }

    #[test]
    fn test_new_invalid_period_zero() {
        let calc = IndicatorCalculator::new(0, 21, 14, 20, 2.0);
        assert!(calc.is_err());
    }

    #[test]
    fn test_first_update_has_no_prev() {
        let mut calc = make_calculator();
        let vals = calc.update(100.0);
        assert!(vals.prev_ema_short.is_none());
        assert!(vals.prev_ema_long.is_none());
        assert_eq!(vals.price, 100.0);
    }

    #[test]
    fn test_second_update_has_prev() {
        let mut calc = make_calculator();
        calc.update(100.0);
        let vals = calc.update(101.0);
        assert!(vals.prev_ema_short.is_some());
        assert!(vals.prev_ema_long.is_some());
        assert_eq!(vals.price, 101.0);
    }

    #[test]
    fn test_ema_short_reacts_faster() {
        let mut calc = make_calculator();
        // Feed stable price then jump
        for _ in 0..50 {
            calc.update(100.0);
        }
        let vals = calc.update(110.0);
        // Short EMA should be closer to 110 than long EMA
        assert!(vals.ema_short > vals.ema_long);
    }

    #[test]
    fn test_bollinger_bands_order() {
        let mut calc = make_calculator();
        for _ in 0..30 {
            calc.update(100.0);
        }
        let vals = calc.update(100.0);
        assert!(vals.bb_upper >= vals.bb_middle);
        assert!(vals.bb_middle >= vals.bb_lower);
    }

    #[test]
    fn test_warm_up() {
        let mut calc = make_calculator();
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64) * 0.1).collect();
        calc.warm_up(&prices);
        let vals = calc.update(105.0);
        assert!(vals.prev_ema_short.is_some());
        assert!(vals.prev_ema_long.is_some());
    }

    #[test]
    fn test_rsi_range() {
        let mut calc = make_calculator();
        // Rising prices → RSI should be high
        for i in 0..30 {
            calc.update(100.0 + i as f64);
        }
        let vals = calc.update(130.0);
        assert!(vals.rsi >= 0.0 && vals.rsi <= 100.0);
    }
}
