use super::fphys as fphys;

pub fn weight(current : fphys, new : fphys, weighting : fphys) -> fphys{
    (current * (weighting - 1.0) + new) / weighting
}
