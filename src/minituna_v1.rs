use ordered_float::OrderedFloat;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::{cell::RefCell, collections::HashMap};

pub struct TrialError {
    message: String,
}

impl TrialError {
    fn new(message: &str) -> TrialError {
        TrialError {
            message: String::from(message),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum TrialState {
    Running,
    Completed,
    Failed,
}

#[derive(Clone)]
pub struct FrozenTrial {
    trial_id: u32,
    state: TrialState,
    value: Option<OrderedFloat<f64>>,
    params: HashMap<String, f64>,
}

impl FrozenTrial {
    pub fn new(trial_id: u32) -> FrozenTrial {
        FrozenTrial {
            trial_id,
            state: TrialState::Running,
            value: None,
            params: HashMap::new(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.state != TrialState::Running
    }
}

#[derive(Clone)]
pub struct Storage {
    trials: Vec<FrozenTrial>,
}

impl Storage {
    pub fn create_new_trial(&mut self) -> u32 {
        let trial_id = self.trials.len() as u32;
        let trial = FrozenTrial::new(trial_id);
        self.trials.push(trial);
        trial_id
    }

    pub fn get_trial(&self, trial_id: u32) -> Option<FrozenTrial> {
        self.trials.get(trial_id as usize).map(|v| v.clone())
    }

    pub fn get_best_trial(&self) -> Option<FrozenTrial> {
        let completed_trials: Vec<FrozenTrial> = self
            .trials
            .iter()
            .filter(|trial| trial.state == TrialState::Completed)
            .map(|v| v.clone())
            .collect();
        let best_trial = completed_trials.into_iter().min_by_key(|t| t.value);
        best_trial
    }

    pub fn set_trial_value(&mut self, trial_id: u32, value: f64) -> Result<(), TrialError> {
        let maybe_trial = self.trials.get_mut(trial_id as usize);
        if let Some(trial) = maybe_trial {
            if !trial.is_finished() {
                return Err(TrialError::new("cannot update finished trial"));
            }
            trial.value = Some(OrderedFloat::from(value)); // TODO いけてんの？？
        }
        Ok(())
    }

    pub fn set_trial_state(&mut self, trial_id: u32, state: TrialState) -> Result<(), TrialError> {
        let maybe_trial = self.trials.get_mut(trial_id as usize);
        if let Some(trial) = maybe_trial {
            if !trial.is_finished() {
                return Err(TrialError::new("cannot update finished trial"));
            }
            trial.state = state;
        }
        Ok(())
    }

    pub fn set_trial_param(
        &mut self,
        trial_id: u32,
        name: &str,
        value: f64,
    ) -> Result<(), TrialError> {
        let maybe_trial = self.trials.get_mut(trial_id as usize);
        if let Some(trial) = maybe_trial {
            if !trial.is_finished() {
                return Err(TrialError::new("cannot update finished trial"));
            }
            trial.params.insert(name.to_string(), value);
        }
        Ok(())
    }
}

pub struct Trial {
    study: RefCell<Study>,
    trial_id: u32,
    state: TrialState,
}

impl Trial {
    pub fn new(trial_id: u32, study: &Study) -> Self {
        Trial {
            study: RefCell::new(study.clone()), // TODO check performance
            trial_id,
            state: TrialState::Running,
        }
    }

    pub fn suggest_uniform(&self, name: &str, low: f64, high: f64) -> Result<f64, TrialError> {
        let maybe_trial = self.study.borrow().storage.get_trial(self.trial_id);
        if let Some(trial) = maybe_trial {
            let mut distribution = HashMap::new();
            distribution.insert(String::from("low"), low);
            distribution.insert(String::from("high"), high);
            let param = self.study.borrow_mut().sampler.sample_independent(
                &self.study.borrow(),
                &trial,
                name,
                distribution,
            );

            match self
                .study
                .borrow_mut()
                .storage
                .set_trial_param(self.trial_id, name, param)
            {
                Ok(_) => Ok(param),
                Err(err) => Err(err),
            }
        } else {
            Err(TrialError::new("Not found specific trial"))
        }
    }
}

#[derive(Clone)]
pub struct Sampler {
    rng: StdRng,
}

impl Sampler {
    pub fn new(seed: u64) -> Self {
        let rng = SeedableRng::seed_from_u64(seed);
        Sampler { rng }
    }

    pub fn sample_independent(
        &mut self,
        _study: &Study,
        _trial: &FrozenTrial,
        _name: &str,
        distribution: HashMap<String, f64>,
    ) -> f64 {
        assert!(distribution.get("low").is_some());
        assert!(distribution.get("high").is_some());
        self.rng.gen_range(
            distribution.get("low").unwrap(),
            distribution.get("high").unwrap(),
        )
    }
}

#[derive(Clone)]
pub struct Study {
    storage: Storage,
    sampler: Sampler,
}

impl Study {
    pub fn optimize<T: Objective>(&mut self, objective: T, n_trials: u32) {
        for _ in 0..n_trials {
            let trial_id = self.storage.create_new_trial();
            let trial = Trial::new(trial_id, self);
            let value = objective.objective(trial);

            let result = value
                .and_then(|v| self.storage.set_trial_value(trial_id, v))
                .and_then(|_| {
                    self.storage
                        .set_trial_state(trial_id, TrialState::Completed)
                });

            match result {
                Ok(()) => (),
                Err(err) => eprintln!("trial_id={} is failed by {}", trial_id, err.message),
            }
        }
    }

    pub fn best_trial(self) -> Option<FrozenTrial> {
        self.storage.get_best_trial()
    }
}

pub trait Objective {
    fn objective(&self, trial: Trial) -> Result<f64, TrialError>;
}
