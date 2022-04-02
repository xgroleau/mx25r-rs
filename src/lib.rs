#![no_std]

pub mod address;
pub mod blocking;
mod command;
pub mod register;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
