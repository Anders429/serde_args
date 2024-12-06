use crate::Container;
use syn::{
    parse_str,
    ItemFn,
};

pub(super) fn expecting(container: &Container) -> ItemFn {
    let descriptions = container.descriptions();
    let mut container_exprs = descriptions
        .container
        .lines
        .into_iter()
        .map(|line| format!("formatter.write_str(\"{line}\")?;"))
        .fold("_ => {".to_owned(), |mut s, line| {
            s.push_str(&line);
            s
        });
    container_exprs.push_str("::std::result::Result::Ok(true)}");
    let key_exprs = descriptions
        .keys
        .into_iter()
        .enumerate()
        .map(|(index, documentation)| {
            let mut documentation_exprs = documentation
                .lines
                .into_iter()
                .map(|line| format!("formatter.write_str(\"{line}\")?;"))
                .fold(
                    format!("::std::option::Option::Some({index}) => {{"),
                    |mut s, line| {
                        s.push_str(&line);
                        s
                    },
                );
            documentation_exprs.push_str("::std::result::Result::Ok(true)}");
            documentation_exprs
        })
        .fold(String::new(), |mut s, expr| {
            s.push_str(&expr);
            s.push('\n');
            s
        });

    parse_str(&format!("
        fn expecting(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {{
            match formatter.width() {{
                {key_exprs}
                {container_exprs}
            }}
        }}
    ")).expect("could not generate help `expecting()` function")
}

#[cfg(test)]
mod tests {
    use super::expecting;
    use claims::assert_ok;
    use syn::{
        parse_str,
        ItemFn,
    };

    #[test]
    fn struct_expecting() {
        assert_eq!(expecting(&assert_ok!(parse_str(
            "
            /// Container documentation.
            struct Foo(
                /// Bar documentation.
                usize,
                /// Baz documentation.
                String
            );"
        ))), assert_ok!(parse_str::<ItemFn>("
            fn expecting(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                match formatter.width() {
                    ::std::option::Option::Some(0) => {
                        formatter.write_str(\"Bar documentation.\")?;
                        ::std::result::Result::Ok(true)
                    }
                    ::std::option::Option::Some(1) => {
                        formatter.write_str(\"Baz documentation.\")?;
                        ::std::result::Result::Ok(true)
                    }
                    _ => {
                        formatter.write_str(\"Container documentation.\")?;
                        ::std::result::Result::Ok(true)
                    }
                }
            }
        ")));
    }

    #[test]
    fn enum_expecting() {
        assert_eq!(expecting(&assert_ok!(parse_str(
            "
            /// Container documentation.
            enum Foo {
                /// Bar documentation.
                Bar,
                /// Baz documentation.
                Baz,
            }"
        ))), assert_ok!(parse_str::<ItemFn>("
            fn expecting(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                match formatter.width() {
                    ::std::option::Option::Some(0) => {
                        formatter.write_str(\"Bar documentation.\")?;
                        ::std::result::Result::Ok(true)
                    }
                    ::std::option::Option::Some(1) => {
                        formatter.write_str(\"Baz documentation.\")?;
                        ::std::result::Result::Ok(true)
                    }
                    _ => {
                        formatter.write_str(\"Container documentation.\")?;
                        ::std::result::Result::Ok(true)
                    }
                }
            }
        ")));
    }
}
