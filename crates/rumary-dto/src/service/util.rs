pub(crate) fn string_to_hash(string: &str) -> crate::service::profile_service::ServiceResult<Vec<u8>> {
    match hex::decode(string) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(Box::new(e)),
    }
}