use syn::{
    parse_str,
    ItemFn,
};

pub(super) fn expecting() -> ItemFn {
    parse_str("
        fn expecting(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
            if formatter.fill() == 'v' {
                formatter.write_str(::std::env!(\"CARGO_PKG_VERSION\"))?;
                ::std::result::Result::Ok(true)
            } else {
                ::std::result::Result::Ok(false)
            }
        }
    ").expect("could not generate version `expecting()` function")
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;
    use syn::{
        parse_str,
        ItemFn,
    };

    #[test]
    fn expecting() {
        assert_eq!(super::expecting(), assert_ok!(parse_str::<ItemFn>("
            fn expecting(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                if formatter.fill() == 'v' {
                    formatter.write_str(::std::env!(\"CARGO_PKG_VERSION\"))?;
                    ::std::result::Result::Ok(true)
                } else {
                    ::std::result::Result::Ok(false)
                }
            }
        ")));
    }
}
