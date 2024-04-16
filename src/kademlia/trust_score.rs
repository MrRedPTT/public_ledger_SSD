const WEIGHT_REPUTATION: f64 = 0.40;
const WEIGHT_RISK: f64 = 0.60;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrustScore {
    reputation: f64,
    risk: f64,
    score: f64,
    total_interactions: i64,
    total_lookups: i64,
    bad_interactions: i64
}


impl TrustScore {
    pub fn new() -> Self {
        TrustScore {
            reputation: 0.0,
            risk: 0.0,
            score: 0.000001, // The default value for when no interaction was measured
            total_interactions: 0,
            total_lookups: 0,
            bad_interactions: 0,
        }
    }

    pub fn bad_reputation(&mut self) {
        if self.total_lookups != 0 {
            self.reputation = self.reputation - (2f64 / self.total_lookups as f64);
        }

    }

    pub fn good_reputation(&mut self) {
        if self.total_lookups == 0 {
            self.reputation = 0.0;
        } else {
            self.reputation += (1f64 / self.total_lookups as f64);
        }

    }

    pub fn bad_interaction(&mut self) {
        self.bad_interactions += 1;
    }

    pub fn new_interaction(&mut self) {
        self.total_interactions += 1;
    }

    fn update_values(&mut self) {
        if self.total_interactions == 0 {
            self.risk = 0.0;
        } else {
            self.risk = self.bad_interactions as f64 / self.total_interactions as f64
        }
    }

    fn update_score(&mut self) {
        self.update_values();
        self.score = WEIGHT_REPUTATION * self.reputation + WEIGHT_RISK * self.risk;
        if self.score == 0.0 || self.score.is_nan(){
            self.score = 0.000001; // Avoid division by 0
        }
    }

    pub fn get_score(&mut self) -> f64 {
        self.update_score();
        return self.score;
    }
}