use crate::trace::{Field, Shape};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    iter,
    iter::Peekable,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    UnexpectedArgument,
    UnrecognizedOption,
    UnrecognizedVariant,
    DuplicateOption,
    InvalidNestedOptional,
    EndOfArgs,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    Identifier(&'static str),
    Value(OsString),
    Context(Context),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Context {
    segments: Vec<Segment>,
}

enum Token {
    Positional(OsString),
    Optional(OsString),
    EndOfOptions,
}

struct ParsedArgs<Args> {
    args: Args,
    revisit: Option<OsString>,
}

impl<Args> ParsedArgs<Args> {
    fn new(args: Args) -> Self {
        Self {
            args,
            revisit: None,
        }
    }
}

impl<Args> ParsedArgs<Args>
where
    Args: Iterator<Item = OsString>,
{
    fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.next() {
            let bytes = token.as_encoded_bytes();
            if let Some(short_bytes) = bytes.strip_prefix(b"-") {
                if short_bytes.is_empty() {
                    // A single `-` is an empty optional token.
                    Some(Token::Optional(OsString::new()))
                } else {
                    if let Some(long_bytes) = short_bytes.strip_prefix(b"-") {
                        if long_bytes.is_empty() {
                            Some(Token::EndOfOptions)
                        } else {
                            Some(Token::Optional(unsafe {
                                OsString::from_encoded_bytes_unchecked(
                                    token.into_encoded_bytes().get_unchecked(2..).to_vec(),
                                )
                            }))
                        }
                    } else {
                        // This is only an option if there is a single character.
                        if short_bytes.len() > 4 {
                            Some(Token::Positional(token))
                        } else {
                            let mut short_bytes_iter = short_bytes.into_iter().copied();
                            let bytes = [
                                short_bytes_iter.next().unwrap_or(0),
                                short_bytes_iter.next().unwrap_or(0),
                                short_bytes_iter.next().unwrap_or(0),
                                short_bytes_iter.next().unwrap_or(0),
                            ];
                            if char::from_u32(u32::from_be_bytes(bytes)).is_some() {
                                Some(Token::Optional(unsafe {
                                    OsString::from_encoded_bytes_unchecked(
                                        token.into_encoded_bytes().get_unchecked(1..).to_vec(),
                                    )
                                }))
                            } else {
                                Some(Token::Positional(token))
                            }
                        }
                    }
                }
            } else {
                Some(Token::Positional(token))
            }
        } else {
            None
        }
    }

    fn next_positional(&mut self) -> Option<OsString> {
        self.next()
    }

    fn next_optional(&mut self) -> Option<OsString> {
        if let Some(token) = self.next_token() {
            match token {
                Token::Optional(token) => Some(token),
                Token::EndOfOptions => None,
                Token::Positional(token) => {
                    self.revisit = Some(token);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl<Args> Iterator for ParsedArgs<Args>
where
    Args: Iterator<Item = OsString>,
{
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.revisit.take().or_else(|| self.args.next())
    }
}

pub(crate) fn parse<Arg, Args>(mut args: Args, shape: &mut Shape) -> Result<Context, Error>
where
    Args: IntoIterator<Item = Arg>,
    Arg: Into<OsString>,
{
    let mut parsed_args = ParsedArgs::new(args.into_iter().map(|arg| arg.into()));
    let result = parse_context_no_options(&mut parsed_args, shape, Context { segments: vec![] })?;
    if parsed_args.next().is_some() {
        Err(Error::UnexpectedArgument)
    } else {
        Ok(result)
    }
}

fn parse_context_no_options<Args>(
    args: &mut ParsedArgs<Args>,
    shape: &mut Shape,
    mut context: Context,
) -> Result<Context, Error>
where
    Args: Iterator<Item = OsString>,
{
    match *shape {
        Shape::Empty => Ok(context),
        Shape::Primitive { .. } => {
            context.segments.push(Segment::Value(
                args.next_positional().ok_or(Error::EndOfArgs)?,
            ));
            Ok(context)
        }
        Shape::Optional(ref mut optional_shape) => {
            // This is a "positional optional". It starts its own isolated context, which only contains its own optional value if it exists.
            match **optional_shape {
                Shape::Empty
                | Shape::Optional(_)
                | Shape::Primitive { .. }
                | Shape::Struct { .. }
                | Shape::Enum { .. } => {
                    if let Some(next) = args.next_positional() {
                        if let Some(next_str) = next.to_str() {
                            match next_str {
                                "-" => {
                                    context.segments.push(Segment::Context(
                                        parse_context_no_options(
                                            args,
                                            optional_shape,
                                            Context { segments: vec![] },
                                        )?,
                                    ));
                                }
                                "--" => {
                                    // End of isolated context.
                                }
                                _ => {
                                    args.revisit = Some(next);
                                }
                            }
                        } else {
                            args.revisit = Some(next);
                        }
                    }
                }
                // Shape::Primitive {..} | Shape::Struct {..} | Shape::Enum {..} => {
                //     if let Some(optional) = args.next_optional() {
                //         let mut optional_context = Context {
                //             segments: vec![],
                //         };
                //         context.segments.push(Segment::Context(parse_context_no_options(&mut ParsedArgs::new(iter::once(optional).chain(args)), optional_shape, optional_context)?));
                //     }
                // }
                Shape::Variant { .. } => {
                    unreachable!()
                }
            }
            Ok(context)
        }
        Shape::Struct {
            ref mut required,
            ref mut optional,
        } => {
            // Parse the struct in its own nested context.
            //
            // While the current context cannot have options, the nested context can.
            let mut end_of_options = false;
            for required_field in required.iter_mut() {
                if end_of_options {
                    match required_field.shape {
                        Shape::Struct { .. } | Shape::Enum { .. } => {
                            let mut inner_context = Context { segments: vec![] };
                            context
                                .segments
                                .push(Segment::Context(parse_context_no_options(
                                    args,
                                    &mut required_field.shape,
                                    inner_context,
                                )?));
                        }
                        Shape::Primitive { .. } | Shape::Empty => {
                            context =
                                parse_context_no_options(args, &mut required_field.shape, context)?;
                        }
                        Shape::Optional(_) | Shape::Variant { .. } => unreachable!(),
                    }
                } else {
                    let parsed_options = match required_field.shape {
                        Shape::Struct { .. } | Shape::Enum { .. } => {
                            let mut inner_context = Context { segments: vec![] };
                            let parsed_context = parse_context(
                                args,
                                &mut required_field.shape,
                                &mut optional
                                    .clone()
                                    .into_iter()
                                    .map(|option| (option, false))
                                    .collect(),
                                inner_context,
                            )?;
                            context
                                .segments
                                .push(Segment::Context(parsed_context.context));
                            end_of_options = parsed_context.closing_end_of_options;
                            parsed_context.options
                        }
                        Shape::Primitive { .. } | Shape::Empty => {
                            let parsed_context = parse_context(
                                args,
                                &mut required_field.shape,
                                &mut optional
                                    .clone()
                                    .into_iter()
                                    .map(|option| (option, false))
                                    .collect(),
                                context,
                            )?;
                            context = parsed_context.context;
                            end_of_options = parsed_context.closing_end_of_options;
                            parsed_context.options
                        }
                        Shape::Optional(_) | Shape::Variant { .. } => unreachable!(),
                    };
                    for (optional_name, optional_context) in parsed_options {
                        let mut found = false;
                        // Find whether the optional name is in this struct.
                        for (index, optional_field) in (&mut *optional).into_iter().enumerate() {
                            if optional_name == optional_field.name
                                || optional_field.aliases.contains(&optional_name)
                            {
                                found = true;
                                context.segments.push(Segment::Context(optional_context));
                                break;
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedOption);
                        }
                    }
                }
            }
            // Parse any remaining options.
            if !end_of_options {
                let parsed_context = parse_context(
                    args,
                    &mut Shape::Empty,
                    &mut optional
                        .clone()
                        .into_iter()
                        .map(|option| (option, false))
                        .collect(),
                    context,
                )?;
                context = parsed_context.context;
                for (optional_name, optional_context) in parsed_context.options {
                    let mut found = false;
                    // Find whether the optional name is in this struct.
                    for optional_field in &mut *optional {
                        if optional_name == optional_field.name
                            || optional_field.aliases.contains(&optional_name)
                        {
                            found = true;
                            context.segments.push(Segment::Context(optional_context));
                            break;
                        }
                    }
                    if !found {
                        return Err(Error::UnrecognizedOption);
                    }
                }
            }

            Ok(context)
        }
        Shape::Enum {
            ref mut variants, ..
        } => {
            let variant_name = args.next_positional().ok_or(Error::EndOfArgs)?;
            let variant_name_str = variant_name.to_str().ok_or(Error::UnrecognizedVariant)?;
            let mut found = false;

            let mut variants_iter = std::mem::take(variants).into_iter();
            let new_shape = loop {
                if let Some(mut variant) = variants_iter.next() {
                    if let Some(static_variant_name) = iter::once(variant.name)
                        .chain(variant.aliases)
                        .find(|s| *s == variant_name_str)
                    {
                        context
                            .segments
                            .push(Segment::Identifier(static_variant_name));
                        context = parse_context_no_options(args, &mut variant.shape, context)?;

                        break Shape::Variant {
                            name: static_variant_name,
                            shape: Box::new(variant.shape),
                        };
                    }
                } else {
                    return Err(Error::UnrecognizedVariant);
                }
            };

            *shape = new_shape;

            if found {
                Ok(context)
            } else {
                Err(Error::UnrecognizedVariant)
            }
        }
        Shape::Variant { .. } => {
            unreachable!()
        }
    }
}

#[derive(Debug)]
struct ParsedContext {
    context: Context,
    options: Vec<(&'static str, Context)>,
    /// If an `EndOfOptions` token appeared at the end of the positional arguments.
    ///
    /// This indicates that the outer context's options should also be terminated.
    closing_end_of_options: bool,
}

fn parse_context<Args>(
    args: &mut ParsedArgs<Args>,
    shape: &mut Shape,
    options: &mut Vec<(Field, bool)>,
    mut context: Context,
) -> Result<ParsedContext, Error>
where
    Args: Iterator<Item = OsString>,
{
    let mut parsed_options = Vec::new();
    let mut closing_end_of_options = false;

    match shape {
        Shape::Empty => {
            while let Some(token) = args.next_token() {
                match token {
                    Token::Positional(value) => {
                        args.revisit = Some(value);
                        break;
                    }
                    Token::Optional(value) => {
                        // Find the option and parse it.
                        let identifier = value.to_str().ok_or(Error::UnrecognizedOption)?;
                        let mut optional_context = Context { segments: vec![] };
                        let mut found = false;
                        let mut index = 0;
                        while index < options.len() {
                            let (optional_field, used) = &options[index];
                            if let Some(static_field_name) = iter::once(optional_field.name)
                                .chain(optional_field.aliases.clone())
                                .find(|s| *s == identifier)
                            {
                                if *used {
                                    index += 1;
                                    continue;
                                }
                                let (mut optional_field, _) = options.remove(index);
                                found = true;
                                optional_context
                                    .segments
                                    .push(Segment::Identifier(static_field_name));
                                let parsed_context = parse_context(
                                    args,
                                    &mut optional_field.shape,
                                    options,
                                    optional_context,
                                )?;
                                parsed_options.extend(parsed_context.options);
                                parsed_options.push((static_field_name, parsed_context.context));
                                if parsed_context.closing_end_of_options {
                                    closing_end_of_options = true;
                                }
                                options.insert(index, (optional_field, true));
                                break;
                            } else {
                                index += 1;
                            }
                        }
                        if !found {
                            // The argument could belong to a neighboring context.
                            args.revisit = Some({
                                let mut bytes = vec![b'-', b'-'];
                                bytes.extend(value.as_encoded_bytes());
                                unsafe { OsString::from_encoded_bytes_unchecked(bytes) }
                            });
                            break;
                        }
                    }
                    Token::EndOfOptions => {
                        closing_end_of_options = true;
                    }
                }
                if closing_end_of_options {
                    break;
                }
            }
        }
        Shape::Primitive { .. } => {
            while let Some(token) = args.next_token() {
                match token {
                    Token::Positional(value) => {
                        context.segments.push(Segment::Value(value));
                        break;
                    }
                    Token::Optional(value) => {
                        let identifier = value.to_str().ok_or(Error::UnrecognizedOption)?;
                        let mut optional_context = Context { segments: vec![] };
                        let mut found = false;
                        let mut index = 0;
                        while index < options.len() {
                            let (optional_field, used) = &options[index];
                            if let Some(static_field_name) = iter::once(optional_field.name)
                                .chain(optional_field.aliases.clone())
                                .find(|s| *s == identifier)
                            {
                                if *used {
                                    index += 1;
                                    continue;
                                }
                                let (mut optional_field, _) = options.remove(index);
                                found = true;
                                optional_context
                                    .segments
                                    .push(Segment::Identifier(static_field_name));
                                let parsed_context = parse_context(
                                    args,
                                    &mut optional_field.shape,
                                    options,
                                    optional_context,
                                )?;
                                parsed_options.extend(parsed_context.options);
                                parsed_options.push((static_field_name, parsed_context.context));
                                if parsed_context.closing_end_of_options {
                                    closing_end_of_options = true;
                                }
                                options.insert(index, (optional_field, true));
                                break;
                            } else {
                                index += 1;
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedOption);
                        }
                    }
                    Token::EndOfOptions => {
                        closing_end_of_options = true;
                    }
                }
                if closing_end_of_options {
                    context = parse_context_no_options(args, shape, context)?;
                    break;
                }
            }
        }
        Shape::Optional(_) => {
            // This is a "positional optional". It starts its own isolated context, which only contains its own optional value if it exists.
            //
            // We therefore simply parse in a no-option context, thereby ignoring all parent context options.
            context = parse_context_no_options(args, shape, context)?;
        }
        Shape::Struct { required, optional } => {
            // Parse the struct in its own nested context.
            let mut end_of_options = false;
            let mut combined_options = options.clone();
            combined_options.extend(optional.clone().into_iter().map(|option| (option, false)));
            for required_field in required.iter_mut() {
                if end_of_options {
                    match required_field.shape {
                        Shape::Struct { .. } | Shape::Enum { .. } => {
                            let mut inner_context = Context { segments: vec![] };
                            context
                                .segments
                                .push(Segment::Context(parse_context_no_options(
                                    args,
                                    &mut required_field.shape,
                                    inner_context,
                                )?));
                        }
                        Shape::Primitive { .. } | Shape::Empty => {
                            context =
                                parse_context_no_options(args, &mut required_field.shape, context)?;
                        }
                        Shape::Optional(_) | Shape::Variant { .. } => unreachable!(),
                    }
                } else {
                    let found_parsed_options = match required_field.shape {
                        Shape::Struct { .. } | Shape::Enum { .. } => {
                            let mut inner_context = Context { segments: vec![] };
                            let parsed_context = parse_context(
                                args,
                                &mut required_field.shape,
                                &mut combined_options,
                                inner_context,
                            )?;
                            context
                                .segments
                                .push(Segment::Context(parsed_context.context));
                            end_of_options = parsed_context.closing_end_of_options;
                            parsed_context.options
                        }
                        Shape::Primitive { .. } | Shape::Empty => {
                            let parsed_context = parse_context(
                                args,
                                &mut required_field.shape,
                                &mut combined_options,
                                context,
                            )?;
                            context = parsed_context.context;
                            end_of_options = parsed_context.closing_end_of_options;
                            parsed_context.options
                        }
                        Shape::Optional(_) | Shape::Variant { .. } => unreachable!(),
                    };
                    'outer: for (optional_name, optional_context) in found_parsed_options {
                        let mut found = false;
                        // Find whether the optional name is in this struct.
                        for optional_field in &mut *optional {
                            if optional_name == optional_field.name
                                || optional_field.aliases.contains(&optional_name)
                            {
                                found = true;
                                context.segments.push(Segment::Context(optional_context));
                                continue 'outer;
                            }
                        }
                        if !found {
                            parsed_options.push((optional_name, optional_context));
                        }
                    }
                }
            }
            // Parse any remaining options.
            if !end_of_options {
                let parsed_context =
                    parse_context(args, &mut Shape::Empty, &mut combined_options, context)?;
                context = parsed_context.context;
                'outer: for (optional_name, optional_context) in parsed_context.options {
                    let mut found = false;
                    // Find whether the optional name is in this struct.
                    for optional_field in &mut *optional {
                        if optional_name == optional_field.name
                            || optional_field.aliases.contains(&optional_name)
                        {
                            found = true;
                            context.segments.push(Segment::Context(optional_context));
                            continue 'outer;
                        }
                    }
                    if !found {
                        parsed_options.push((optional_name, optional_context));
                    }
                }
                if parsed_context.closing_end_of_options {
                    closing_end_of_options = true;
                }
            }

            // Mark all found options as being found.
            for ((_, found), (_, combined_found)) in options.iter_mut().zip(combined_options) {
                *found = combined_found;
            }
        }
        Shape::Enum { variants, .. } => {
            // Parse the variant.
            while let Some(token) = args.next_token() {
                match token {
                    Token::Positional(variant_name) => {
                        let variant_name_str =
                            variant_name.to_str().ok_or(Error::UnrecognizedVariant)?;
                        let mut found = false;
                        for mut variant in std::mem::take(variants) {
                            if let Some(static_variant_name) = iter::once(variant.name)
                                .chain(variant.aliases)
                                .find(|s| *s == variant_name_str)
                            {
                                context
                                    .segments
                                    .push(Segment::Identifier(static_variant_name));
                                // Parse the variant's shape.
                                let parsed_context =
                                    parse_context(args, &mut variant.shape, options, context)?;
                                context = parsed_context.context;
                                // Handle options.
                                parsed_options.extend(parsed_context.options);
                                if parsed_context.closing_end_of_options {
                                    closing_end_of_options = true;
                                }
                                // Update shape.
                                *shape = Shape::Variant {
                                    name: static_variant_name,
                                    shape: Box::new(variant.shape),
                                };
                                found = true;
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedVariant);
                        }
                        break;
                    }
                    Token::Optional(value) => {
                        let identifier = value.to_str().ok_or(Error::UnrecognizedOption)?;
                        let mut optional_context = Context { segments: vec![] };
                        let mut found = false;
                        let mut index = 0;
                        while index < options.len() {
                            let (optional_field, used) = &options[index];
                            if let Some(static_field_name) = iter::once(optional_field.name)
                                .chain(optional_field.aliases.clone())
                                .find(|s| *s == identifier)
                            {
                                if *used {
                                    index += 1;
                                    continue;
                                }
                                let (mut optional_field, _) = options.remove(index);
                                found = true;
                                optional_context
                                    .segments
                                    .push(Segment::Identifier(static_field_name));
                                let parsed_context = parse_context(
                                    args,
                                    &mut optional_field.shape,
                                    options,
                                    optional_context,
                                )?;
                                parsed_options.extend(parsed_context.options);
                                parsed_options.push((static_field_name, parsed_context.context));
                                if parsed_context.closing_end_of_options {
                                    closing_end_of_options = true;
                                }
                                options.insert(index, (optional_field, true));
                                break;
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedOption);
                        }
                    }
                    Token::EndOfOptions => {
                        let variant_name = args.next_positional().ok_or(Error::EndOfArgs)?;
                        let variant_name_str =
                            variant_name.to_str().ok_or(Error::UnrecognizedVariant)?;
                        let mut found = false;
                        for mut variant in std::mem::take(variants) {
                            if let Some(static_variant_name) = iter::once(variant.name)
                                .chain(variant.aliases)
                                .find(|s| *s == variant_name_str)
                            {
                                context
                                    .segments
                                    .push(Segment::Identifier(static_variant_name));
                                context =
                                    parse_context_no_options(args, &mut variant.shape, context)?;

                                found = true;

                                *shape = Shape::Variant {
                                    name: static_variant_name,
                                    shape: Box::new(variant.shape),
                                };
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedVariant);
                        }
                        break;
                    }
                }
            }
        }
        Shape::Variant { .. } => unreachable!(),
    }

    Ok(ParsedContext {
        context,
        options: parsed_options,
        closing_end_of_options,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse, Context, Error, Segment};
    use crate::trace::{Field, Shape};
    use claims::{assert_err_eq, assert_ok_eq};

    #[test]
    fn parse_empty() {
        assert_ok_eq!(
            parse(Vec::<&str>::new(), &mut Shape::Empty),
            Context {
                segments: Vec::new(),
            }
        );
    }

    #[test]
    fn parse_primitive() {
        assert_ok_eq!(
            parse(
                ["foo"],
                &mut Shape::Primitive {
                    name: "bar".to_owned()
                }
            ),
            Context {
                segments: vec![Segment::Value("foo".into())],
            }
        );
    }

    #[test]
    fn parse_primitive_end_of_args() {
        assert_err_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Primitive {
                    name: "bar".to_owned()
                }
            ),
            Error::EndOfArgs
        );
    }

    #[test]
    fn parse_optional_primitive() {
        assert_ok_eq!(
            parse(
                ["-", "foo"],
                &mut Shape::Optional(Box::new(Shape::Primitive {
                    name: "bar".to_owned()
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("foo".into())]
                })],
            }
        );
    }

    #[test]
    fn parse_struct_empty() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Struct {
                    required: vec![],
                    optional: vec![],
                }
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_struct_single_field() {
        assert_ok_eq!(
            parse(
                vec!["foo"],
                &mut Shape::Struct {
                    required: vec![Field {
                        name: "bar",
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        }
                    }],
                    optional: vec![],
                }
            ),
            Context {
                segments: vec![Segment::Value("foo".into()),]
            }
        );
    }

    #[test]
    fn parse_struct_multiple_fields() {
        assert_ok_eq!(
            parse(
                vec!["foo", "bar"],
                &mut Shape::Struct {
                    required: vec![
                        Field {
                            name: "baz",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned()
                            }
                        },
                        Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned()
                            }
                        }
                    ],
                    optional: vec![],
                }
            ),
            Context {
                segments: vec![Segment::Value("foo".into()), Segment::Value("bar".into()),]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_struct_single_option_present() {
        assert_ok_eq!(
            parse(
                vec!["--bar", "foo"],
                &mut Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("bar"), Segment::Value("foo".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_present_alias() {
        assert_ok_eq!(
            parse(
                vec!["--qux", "foo"],
                &mut Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec!["qux"],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("qux"), Segment::Value("foo".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_present_multiple_aliases() {
        assert_err_eq!(
            parse(
                vec!["--qux", "foo", "--bar", "baz"],
                &mut Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec!["qux"],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Error::UnexpectedArgument,
        );
    }

    #[test]
    fn parse_struct_mixed_fields() {
        assert_ok_eq!(
            parse(
                vec!["123", "--bar", "foo", "456", "--qux", "789"],
                &mut Shape::Struct {
                    required: vec![
                        Field {
                            name: "foo",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "quux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "bar",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "missing",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            }
                        },
                    ]
                }
            ),
            Context {
                segments: vec![
                    Segment::Value("123".into()),
                    Segment::Value("456".into()),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("foo".into())],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("qux"), Segment::Value("789".into())],
                    }),
                ]
            }
        );
    }

    #[test]
    fn parse_struct_nested() {
        assert_ok_eq!(
            parse(
                vec!["123", "--bar", "foo", "--qux", "789", "456"],
                &mut Shape::Struct {
                    required: vec![
                        Field {
                            name: "inner_struct",
                            aliases: vec![],
                            shape: Shape::Struct {
                                required: vec![Field {
                                    name: "foo",
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned()
                                    },
                                },],
                                optional: vec![Field {
                                    name: "bar",
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned()
                                    },
                                },],
                            }
                        },
                        Field {
                            name: "quux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "missing",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                    ],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![
                            Segment::Value("123".into()),
                            Segment::Context(Context {
                                segments: vec![
                                    Segment::Identifier("bar"),
                                    Segment::Value("foo".into()),
                                ]
                            }),
                        ],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("qux"), Segment::Value("789".into()),],
                    }),
                    Segment::Value("456".into()),
                ]
            }
        );
    }
}
