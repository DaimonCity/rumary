// pub struct Level(u16);
//
// impl Level {
//     pub fn new(val: u16) -> Result<Self, AccessError> {
//         if val > 100 { return Err(AccessError::TooHigh); }
//         Ok(Self(val))
//     }
// }
//
// pub struct AccessLevel {
//     pub role_type: RoleType,
//     pub level: Level, // Теперь нельзя создать AccessLevel с невалидным числом
// }
//
// pub enum AccessError {
//     TooHigh,
// }
