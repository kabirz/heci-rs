pub struct Heci {
    pub device: isize,
}
pub const UVSS_GUID: &str = "4f806aa7-9f6f-4662-90e5-3f26ed87c58f";
pub const HECI_TEST: &str = "a6bd915c-fe11-49f1-81c0-74b629f289ac";
pub trait HeciOp {
    fn connect(&self, guid: &str) -> i32;
    fn write(&self, data: &[u8]) -> i32;
    fn read(&self, data: &mut [u8]) -> i32;
    fn close(&self);
}

#[cfg(windows)]
mod heci_windows;

#[cfg(unix)]
mod heci_linux;

pub fn add(left: usize, right: usize) -> usize {
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
