use clmm_lp_domain::value_objects::amount::Amount;

/// Trait for modeling volume.
pub trait VolumeModel {
    /// Returns the volume for the next step.
    fn next_volume(&mut self) -> Amount;
}

/// Constant volume model.
#[derive(Clone)]
pub struct ConstantVolume {
    /// The constant volume amount.
    pub amount: Amount,
}

impl VolumeModel for ConstantVolume {
    fn next_volume(&mut self) -> Amount {
        self.amount
    }
}

// Could add StochasticVolume later
