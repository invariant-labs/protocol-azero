use math::types::percentage::Percentage;
#[derive(PartialEq, Default, Debug)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}
