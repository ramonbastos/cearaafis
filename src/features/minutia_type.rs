/// Minutia type enum — mirrors .NET MinutiaType.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinutiaType {
    Ending = 0,
    Bifurcation = 1,
}
