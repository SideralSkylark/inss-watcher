#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub cents: i64,
}

impl Money {
    pub fn from_str(raw: &str) -> Option<Self> {
        let (left, right) = raw.split_once(',')
            .unwrap_or((raw, "0"));

        let whole: i64 = left.replace('.', "").parse().ok()?;

        let cents: i64 = match right.len() {
            0 => 0,
            1 => right.parse::<i64>().ok()? * 10,  // turbofish before (), not after
            2 => right.parse::<i64>().ok()?,
            _ => return None,
        };

        Some(Self {
            cents: whole * 100 + cents,  // was value2, now cents
        })
    }
}
