use math::types::percentage::Percentage;
#[derive(PartialEq, Default, Debug, scale::Encode, scale::Decode)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}
