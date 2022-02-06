use nanoid::nanoid;

pub const DEFAULT_UID_CHARS: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

pub fn create_uid(size: usize, chars: &Vec<char>) -> String {
    let uid = nanoid!(size, chars);
    uid
}

#[macro_export]
macro_rules! uid {
    ($size: tt, $chars: expr) => {
        $crate::utils::uid::create_uid($size, $chars)
    };
    ($size: tt) => {
        $crate::utils::uid::create_uid($size, &$crate::utils::uid::DEFAULT_UID_CHARS.to_vec())
    };
    () => {
        $crate::utils::uid::create_uid(21, &$crate::utils::uid::DEFAULT_UID_CHARS.to_vec())
    };
}

#[cfg(test)]
mod utils_tests {
    #[test]
    fn should_create_uid() {
        assert_eq!(uid!().len(), 21);
        assert_eq!(uid!(10).len(), 10);
        assert_eq!(uid!(5).len(), 5);
    }
}
