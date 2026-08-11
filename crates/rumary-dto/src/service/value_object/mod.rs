/// Конверсии между value object-ами разных доменов имеют смысл только когда
/// оба домена включены.
#[cfg(all(feature = "domain_api", feature = "domain_perms"))]
pub mod user_id;
