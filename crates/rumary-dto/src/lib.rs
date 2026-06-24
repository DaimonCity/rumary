pub mod domain;
pub mod dto;
pub mod service;
mod external;

#[cfg(feature = "mojang")]
pub use {
    external::mojang
};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
