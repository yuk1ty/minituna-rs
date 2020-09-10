use crate::minituna_v1::Objective;
use crate::minituna_v1::Trial;
use crate::minituna_v1::TrialError;

struct Quadratic;

impl Objective for Quadratic {
    fn objective(&self, trial: Trial) -> Result<f64, TrialError> {
        let x = trial.suggest_uniform("x", 0.0, 10.0);
        let y = trial.suggest_uniform("y", 0.0, 10.0);
        match (x, y) {
            (Ok(x1), Ok(y1)) => Ok((x1 - 3.0).powi(2) + (y1 - 5.0).powi(2)),
            (_, Err(ey1)) => Err(ey1),
            (Err(ex1), _) => Err(ex1),
        }
    }
}
