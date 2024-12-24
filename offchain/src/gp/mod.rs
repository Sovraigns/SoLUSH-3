pub mod population;
pub mod genetic_ops;
// pub mod fitness;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy() {
        assert_eq!(2+2, 4);
    }
}
