use cfg_if::cfg_if;
use nanoid::nanoid;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

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
        $crate::utils::create_uid($size, $chars)
    };
    ($size: tt) => {
        $crate::utils::create_uid($size, &$crate::utils::DEFAULT_UID_CHARS.to_vec())
    };
    () => {
        $crate::utils::create_uid(21, &$crate::utils::DEFAULT_UID_CHARS.to_vec())
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
