macro_rules! wrap_error {
    ($src_type:ty, $dest_type:ident :: $variant:ident) => {
        impl From<$src_type> for $dest_type {
            fn from(err: $src_type) -> $dest_type {
                $dest_type::$variant(err)
            }
        }
    }
}
