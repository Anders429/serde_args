mod levenshtein;

use crate::trace::{Field, Shape};
use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
    iter, str, vec,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    MissingArguments(Vec<String>),
    UnexpectedArgument(Vec<u8>),
    UnrecognizedOption {
        name: String,
        expecting: Vec<&'static str>,
    },
    UnrecognizedVariant {
        name: String,
        expecting: Vec<&'static str>,
    },
    Help,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingArguments(arguments) => {
                if arguments.len() == 1 {
                    write!(
                        formatter,
                        "missing required positional argument: <{}>",
                        arguments.last().expect("argument not present")
                    )
                } else {
                    formatter.write_str("missing required posiitonal arguments:")?;
                    for argument in arguments {
                        write!(formatter, " <{}>", argument)?;
                    }
                    Ok(())
                }
            }
            Self::UnexpectedArgument(argument) => {
                write!(
                    formatter,
                    "unexpected positional argument: {}",
                    String::from_utf8_lossy(&argument)
                )
            }
            Self::UnrecognizedOption { name, expecting } => {
                // Find the most similar option.
                let hint = expecting
                    .iter()
                    .map(|field| (field, levenshtein::distance(name, field)))
                    .filter(|(_, distance)| *distance < 5)
                    .min_by_key(|(_, distance)| *distance)
                    .map(|(name, _)| name);
                // Write message.
                write!(
                    formatter,
                    "unrecognized optional flag: {}",
                    if name.chars().count() == 1 {
                        format!("-{}", name)
                    } else {
                        format!("--{}", name)
                    }
                )?;
                if let Some(field) = hint {
                    write!(
                        formatter,
                        "\n\n  tip: a similar option exists: {}",
                        if field.chars().count() == 1 {
                            format!("-{}", field)
                        } else {
                            format!("--{}", field)
                        },
                    )?;
                }
                Ok(())
            }
            Self::UnrecognizedVariant { name, expecting } => {
                // Find the most similar command.
                let hint = expecting
                    .iter()
                    .map(|variant| (variant, levenshtein::distance(name, variant)))
                    .filter(|(_, distance)| *distance < 5)
                    .min_by_key(|(_, distance)| *distance)
                    .map(|(name, _)| name);
                // Write message.
                write!(formatter, "unrecognized command: {}", name)?;
                if let Some(variant) = hint {
                    write!(
                        formatter,
                        "\n\n  tip: a similar command exists: {}",
                        variant
                    )?;
                }
                Ok(())
            }
            Self::Help => formatter.write_str("help requested"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    Identifier(&'static str),
    Value(Vec<u8>),
    Context(Context),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Context {
    pub(crate) segments: Vec<Segment>,
}

impl IntoIterator for Context {
    type IntoIter = ContextIter;
    type Item = Segment;

    fn into_iter(self) -> Self::IntoIter {
        ContextIter {
            segments: self.segments.into_iter(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ContextIter {
    segments: vec::IntoIter<Segment>,
}

impl Iterator for ContextIter {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        self.segments.next()
    }
}

enum Token {
    Positional(Vec<u8>),
    Optional(Vec<u8>),
    EndOfOptions,
}

struct ParsedArgs<Args> {
    args: Args,
    revisit: Option<Vec<u8>>,
    consumed_token: bool,
}

impl<Args> ParsedArgs<Args> {
    fn new(args: Args) -> Self {
        Self {
            args,
            revisit: None,
            consumed_token: false,
        }
    }
}

impl<Args> ParsedArgs<Args>
where
    Args: Iterator<Item = OsString>,
{
    fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.next() {
            if let Some(short_token) = token.strip_prefix(b"-") {
                if short_token.is_empty() {
                    // A single `-` is an empty optional token.
                    Some(Token::Optional(Vec::new()))
                } else {
                    if let Some(long_token) = short_token.strip_prefix(b"-") {
                        if long_token.is_empty() {
                            Some(Token::EndOfOptions)
                        } else {
                            Some(Token::Optional(long_token.to_vec()))
                        }
                    } else {
                        // This is only an option if there is a single character.
                        if short_token.len() > 4 {
                            Some(Token::Positional(token))
                        } else {
                            if let Ok(short_token_str) = str::from_utf8(short_token) {
                                if short_token_str.chars().count() == 1 {
                                    Some(Token::Optional(short_token.to_vec()))
                                } else {
                                    Some(Token::Positional(token))
                                }
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

    fn next_positional(&mut self) -> Option<Vec<u8>> {
        self.next()
    }

    fn next_optional(&mut self) -> Option<Vec<u8>> {
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
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self
            .revisit
            .take()
            .or_else(|| self.args.next().map(|os_str| os_str.into_encoded_bytes()));
        if value.is_some() {
            self.consumed_token = true;
        }
        value
    }
}

pub(crate) fn parse<Arg, Args>(args: Args, shape: &mut Shape) -> Result<Context, Error>
where
    Args: IntoIterator<Item = Arg>,
    Arg: Into<OsString>,
{
    let mut parsed_args = ParsedArgs::new(args.into_iter().map(|arg| arg.into()));
    let parsed_context = parse_context(
        &mut parsed_args,
        shape,
        &mut vec![Field {
            name: "help",
            description: "Display this message.".into(),
            aliases: vec!["h"],
            shape: Shape::Empty {
                description: String::new(),
            },
        }],
        Context { segments: vec![] },
    );

    // Handle overriding options.
    for (option_name, _option_context) in parsed_context.options {
        match option_name {
            "help" | "h" => return Err(Error::Help),
            _ => {
                return Err(Error::UnrecognizedOption {
                    name: option_name.to_owned(),
                    expecting: vec!["help", "h"],
                })
            }
        }
    }

    let context = parsed_context.context.map_err(|error| {
        if matches!(error, Error::MissingArguments(_)) && !parsed_args.consumed_token {
            Error::Help
        } else {
            error
        }
    })?;

    // Ensure there are no remaining arguments.
    let mut end_of_options = parsed_context.closing_end_of_options;
    loop {
        if end_of_options {
            if let Some(value) = parsed_args.next_positional() {
                return Err(Error::UnexpectedArgument(value));
            } else {
                break;
            }
        } else {
            if let Some(token) = parsed_args.next_token() {
                match token {
                    Token::Positional(value) => {
                        return Err(Error::UnexpectedArgument(value));
                    }
                    Token::Optional(value) => {
                        return Err(Error::UnrecognizedOption {
                            name: String::from_utf8_lossy(&value).into(),
                            expecting: vec!["help", "h"]
                                .into_iter()
                                .chain(shape.trailing_options().into_iter().flat_map(|field| {
                                    iter::once(field.name).chain(field.aliases.iter().copied())
                                }))
                                .collect(),
                        });
                    }
                    Token::EndOfOptions => {
                        end_of_options = true;
                    }
                }
            } else {
                break;
            }
        }
    }

    Ok(context)
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
        Shape::Empty { .. } => Ok(context),
        Shape::Primitive { ref name, .. } => {
            context.segments.push(Segment::Value(
                args.next_positional()
                    .ok_or(Error::MissingArguments(vec![name.clone()]))?,
            ));
            Ok(context)
        }
        Shape::Optional(ref mut optional_shape) => {
            // This is a "positional optional". It starts its own isolated context, which only contains its own optional value if it exists.
            match **optional_shape {
                Shape::Empty { .. } | Shape::Optional(_) => {
                    if let Some(next) = args.next_positional() {
                        if let Ok(next_str) = str::from_utf8(&next) {
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
                Shape::Primitive { .. } | Shape::Struct { .. } | Shape::Enum { .. } => {
                    if let Some(optional) = args.next_optional() {
                        args.revisit = Some(optional);
                        let optional_context = Context { segments: vec![] };
                        context
                            .segments
                            .push(Segment::Context(parse_context_no_options(
                                args,
                                optional_shape,
                                optional_context,
                            )?));
                    }
                }
                Shape::Variant { .. } => {
                    unreachable!()
                }
            }
            Ok(context)
        }
        Shape::Struct {
            ref mut required,
            ref mut optional,
            ..
        } => {
            // Parse the struct in its own nested context.
            //
            // While the current context cannot have options, the nested context can.
            let mut end_of_options = false;
            let mut required_iter = required.iter_mut();
            while let Some(required_field) = required_iter.next() {
                let inner_context = Context {
                    segments: vec![Segment::Identifier(required_field.name)],
                };
                if end_of_options {
                    context.segments.push(Segment::Context(
                        match parse_context_no_options(
                            args,
                            &mut required_field.shape,
                            inner_context,
                        ) {
                            Ok(context) => context,
                            Err(error) => {
                                return Err({
                                    if let Error::MissingArguments(mut arguments) = error {
                                        // Replace the last argument if it was primitive.
                                        if arguments.len() == 1
                                            && matches!(
                                                required_field.shape,
                                                Shape::Primitive { .. } | Shape::Enum { .. }
                                            )
                                        {
                                            *arguments.last_mut().expect("no arguments") =
                                                required_field.name.to_owned();
                                        }
                                        // Append any more missing arguments.
                                        arguments.extend(
                                            required_iter.map(|field| field.name.to_owned()),
                                        );
                                        Error::MissingArguments(arguments)
                                    } else {
                                        error
                                    }
                                });
                            }
                        },
                    ));
                } else {
                    let parsed_context = parse_context(
                        args,
                        &mut required_field.shape,
                        &mut optional.clone(),
                        inner_context,
                    );
                    context
                        .segments
                        .push(Segment::Context(match parsed_context.context {
                            Ok(context) => context,
                            Err(error) => {
                                return Err({
                                    if let Error::MissingArguments(mut arguments) = error {
                                        // Replace the last argument if it was primitive.
                                        if arguments.len() == 1
                                            && matches!(
                                                required_field.shape,
                                                Shape::Primitive { .. } | Shape::Enum { .. }
                                            )
                                        {
                                            *arguments.last_mut().expect("no arguments") =
                                                required_field.name.to_owned();
                                        }
                                        // Append any more missing arguments.
                                        arguments.extend(
                                            required_iter.map(|field| field.name.to_owned()),
                                        );
                                        Error::MissingArguments(arguments)
                                    } else {
                                        error
                                    }
                                });
                            }
                        }));
                    end_of_options = parsed_context.closing_end_of_options;
                    let parsed_options = parsed_context.options;
                    for (optional_name, optional_context) in parsed_options {
                        let mut found = false;
                        // Find whether the optional name is in this struct.
                        for optional_field in (&mut *optional).into_iter() {
                            if optional_name == optional_field.name
                                || optional_field.aliases.contains(&optional_name)
                            {
                                found = true;
                                context.segments.push(Segment::Context(optional_context));
                                break;
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedOption {
                                name: optional_name.into(),
                                expecting: optional
                                    .iter()
                                    .map(|field| {
                                        iter::once(field.name).chain(field.aliases.iter().copied())
                                    })
                                    .flatten()
                                    .collect(),
                            });
                        }
                    }
                }
            }
            // Parse any remaining options.
            if !end_of_options {
                let parsed_context = parse_context(
                    args,
                    &mut Shape::Empty {
                        description: String::new(),
                    },
                    &mut optional.clone(),
                    context,
                );
                context = parsed_context.context?;
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
                        return Err(Error::UnrecognizedOption {
                            name: optional_name.into(),
                            expecting: optional
                                .iter()
                                .map(|field| {
                                    iter::once(field.name).chain(field.aliases.iter().copied())
                                })
                                .flatten()
                                .collect(),
                        });
                    }
                }
            }

            Ok(context)
        }
        Shape::Enum {
            name,
            ref mut variants,
            ..
        } => {
            let variant_name = args
                .next_positional()
                .ok_or(Error::MissingArguments(vec![name.into()]))?;
            let variant_name_str = str::from_utf8(&variant_name).or_else(|_| {
                Err(Error::UnrecognizedVariant {
                    name: String::from_utf8_lossy(&variant_name).into(),
                    expecting: variants
                        .iter()
                        .map(|variant| {
                            iter::once(variant.name).chain(variant.aliases.iter().copied())
                        })
                        .flatten()
                        .collect(),
                })
            })?;

            let mut variants_iter = variants.clone().into_iter();
            loop {
                if let Some(variant) = variants_iter.next() {
                    if let Some(static_variant_name) = iter::once(variant.name)
                        .chain(variant.aliases)
                        .find(|s| *s == variant_name_str)
                    {
                        *shape = Shape::Variant {
                            name: static_variant_name,
                            shape: Box::new(variant.shape),
                            description: variant.description,
                            enum_name: name,
                            variants: variants.clone(),
                        };
                        if let Shape::Variant {
                            shape: inner_shape, ..
                        } = shape
                        {
                            context
                                .segments
                                .push(Segment::Identifier(static_variant_name));
                            context = parse_context_no_options(args, inner_shape, context)?;
                        } else {
                            unreachable!();
                        }

                        return Ok(context);
                    }
                } else {
                    return Err(Error::UnrecognizedVariant {
                        name: variant_name_str.into(),
                        expecting: variants
                            .iter()
                            .map(|variant| {
                                iter::once(variant.name).chain(variant.aliases.iter().copied())
                            })
                            .flatten()
                            .collect(),
                    });
                }
            }
        }
        Shape::Variant {
            enum_name,
            ref mut variants,
            ..
        } => {
            let variant_name = args
                .next_positional()
                .ok_or(Error::MissingArguments(vec![enum_name.into()]))?;
            let variant_name_str = str::from_utf8(&variant_name).or_else(|_| {
                Err(Error::UnrecognizedVariant {
                    name: String::from_utf8_lossy(&variant_name).into(),
                    expecting: variants
                        .iter()
                        .map(|variant| {
                            iter::once(variant.name).chain(variant.aliases.iter().copied())
                        })
                        .flatten()
                        .collect(),
                })
            })?;

            for variant in variants.iter_mut() {
                if let Some(static_variant_name) = iter::once(variant.name)
                    .chain(variant.aliases.clone())
                    .find(|s| *s == variant_name_str)
                {
                    context
                        .segments
                        .push(Segment::Identifier(static_variant_name));
                    return parse_context_no_options(args, &mut variant.shape, context);
                }
            }

            Err(Error::UnrecognizedVariant {
                name: variant_name_str.into(),
                expecting: variants
                    .iter()
                    .map(|variant| iter::once(variant.name).chain(variant.aliases.iter().copied()))
                    .flatten()
                    .collect(),
            })
        }
    }
}

#[derive(Debug)]
struct ParsedContext {
    context: Result<Context, Error>,
    options: Vec<(&'static str, Context)>,
    /// If an `EndOfOptions` token appeared at the end of the positional arguments.
    ///
    /// This indicates that the outer context's options should also be terminated.
    closing_end_of_options: bool,
}

fn parse_context<Args>(
    args: &mut ParsedArgs<Args>,
    shape: &mut Shape,
    options: &mut Vec<Field>,
    mut context: Context,
) -> ParsedContext
where
    Args: Iterator<Item = OsString>,
{
    let mut parsed_options = Vec::new();
    let mut closing_end_of_options = false;

    // This is wrapped in a function to allow easily returning errors with `?` while also giving
    // context to the outer scope.
    let context = (|| {
        match shape {
            Shape::Empty { .. } => {
                while let Some(token) = args.next_token() {
                    match token {
                        Token::Positional(value) => {
                            args.revisit = Some(value);
                            break;
                        }
                        Token::Optional(value) => {
                            // Find the option and parse it.
                            let identifier =
                                str::from_utf8(&value).or(Err(Error::UnrecognizedOption {
                                    name: String::from_utf8_lossy(&value).into(),
                                    expecting: options
                                        .iter()
                                        .map(|field| {
                                            iter::once(field.name)
                                                .chain(field.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                }))?;
                            let mut optional_context = Context { segments: vec![] };
                            let mut found = false;
                            let mut index = 0;
                            while index < options.len() {
                                let optional_field = &options[index];
                                if let Some(static_field_name) = iter::once(optional_field.name)
                                    .chain(optional_field.aliases.clone())
                                    .find(|s| *s == identifier)
                                {
                                    let mut optional_field = options.remove(index);
                                    found = true;
                                    optional_context
                                        .segments
                                        .push(Segment::Identifier(static_field_name));
                                    let parsed_context = parse_context(
                                        args,
                                        &mut optional_field.shape,
                                        options,
                                        optional_context,
                                    );
                                    parsed_options.extend(parsed_context.options);
                                    parsed_options
                                        .push((static_field_name, parsed_context.context?));
                                    if parsed_context.closing_end_of_options {
                                        closing_end_of_options = true;
                                    }
                                    options.insert(index, optional_field);
                                    break;
                                } else {
                                    index += 1;
                                }
                            }
                            if !found {
                                // The argument could belong to a neighboring context.
                                args.revisit = Some({
                                    let mut bytes = vec![b'-', b'-'];
                                    bytes.extend(value);
                                    bytes
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
            Shape::Primitive { ref name, .. } => loop {
                let token = args
                    .next_token()
                    .ok_or(Error::MissingArguments(vec![name.clone()]))?;
                match token {
                    Token::Positional(value) => {
                        context.segments.push(Segment::Value(value));
                        break;
                    }
                    Token::Optional(value) => {
                        let identifier =
                            str::from_utf8(&value).or(Err(Error::UnrecognizedOption {
                                name: String::from_utf8_lossy(&value).into(),
                                expecting: options
                                    .iter()
                                    .map(|field| {
                                        iter::once(field.name).chain(field.aliases.iter().copied())
                                    })
                                    .flatten()
                                    .collect(),
                            }))?;
                        let mut optional_context = Context { segments: vec![] };
                        let mut found = false;
                        let mut index = 0;
                        while index < options.len() {
                            let optional_field = &options[index];
                            if let Some(static_field_name) = iter::once(optional_field.name)
                                .chain(optional_field.aliases.clone())
                                .find(|s| *s == identifier)
                            {
                                let mut optional_field = options.remove(index);
                                found = true;
                                optional_context
                                    .segments
                                    .push(Segment::Identifier(static_field_name));
                                let parsed_context = parse_context(
                                    args,
                                    &mut optional_field.shape,
                                    options,
                                    optional_context,
                                );
                                parsed_options.extend(parsed_context.options);
                                parsed_options.push((static_field_name, parsed_context.context?));
                                if parsed_context.closing_end_of_options {
                                    closing_end_of_options = true;
                                }
                                options.insert(index, optional_field);
                                break;
                            } else {
                                index += 1;
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedOption {
                                name: identifier.into(),
                                expecting: options
                                    .iter()
                                    .map(|field| {
                                        iter::once(field.name).chain(field.aliases.iter().copied())
                                    })
                                    .flatten()
                                    .collect(),
                            });
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
            },
            Shape::Optional(_) => {
                // This is a "positional optional". It starts its own isolated context, which only contains its own optional value if it exists.
                //
                // We therefore simply parse in a no-option context, thereby ignoring all parent context options.
                context = parse_context_no_options(args, shape, context)?;
            }
            Shape::Struct {
                required, optional, ..
            } => {
                // Parse the struct in its own nested context.
                let mut end_of_options = false;
                let mut combined_options = options.clone();
                combined_options.extend(optional.clone());
                let mut required_iter = required.iter_mut();
                while let Some(required_field) = required_iter.next() {
                    let inner_context = Context {
                        segments: vec![Segment::Identifier(required_field.name)],
                    };
                    if end_of_options {
                        context.segments.push(Segment::Context(
                            match parse_context_no_options(
                                args,
                                &mut required_field.shape,
                                inner_context,
                            ) {
                                Ok(context) => context,
                                Err(error) => {
                                    return Err({
                                        if let Error::MissingArguments(mut arguments) = error {
                                            // Replace the last argument if it was primitive.
                                            if arguments.len() == 1
                                                && matches!(
                                                    required_field.shape,
                                                    Shape::Primitive { .. } | Shape::Enum { .. }
                                                )
                                            {
                                                *arguments.last_mut().expect("no arguments") =
                                                    required_field.name.to_owned();
                                            }
                                            // Append any more missing arguments.
                                            arguments.extend(
                                                required_iter.map(|field| field.name.to_owned()),
                                            );
                                            Error::MissingArguments(arguments)
                                        } else {
                                            error
                                        }
                                    });
                                }
                            },
                        ));
                    } else {
                        let parsed_context = parse_context(
                            args,
                            &mut required_field.shape,
                            &mut combined_options,
                            inner_context,
                        );
                        end_of_options = parsed_context.closing_end_of_options;
                        let found_parsed_options = parsed_context.options;
                        'outer: for (optional_name, optional_context) in found_parsed_options {
                            // Find whether the optional name is in this struct.
                            for optional_field in &mut *optional {
                                if optional_name == optional_field.name
                                    || optional_field.aliases.contains(&optional_name)
                                {
                                    context.segments.push(Segment::Context(optional_context));
                                    continue 'outer;
                                }
                            }
                            parsed_options.push((optional_name, optional_context));
                        }
                        context
                            .segments
                            .push(Segment::Context(match parsed_context.context {
                                Ok(context) => context,
                                Err(error) => {
                                    return Err({
                                        if let Error::MissingArguments(mut arguments) = error {
                                            // Replace the last argument if it was primitive.
                                            if arguments.len() == 1
                                                && matches!(
                                                    required_field.shape,
                                                    Shape::Primitive { .. } | Shape::Enum { .. }
                                                )
                                            {
                                                *arguments.last_mut().expect("no arguments") =
                                                    required_field.name.to_owned();
                                            }
                                            // Append any more missing arguments.
                                            arguments.extend(
                                                required_iter.map(|field| field.name.to_owned()),
                                            );
                                            Error::MissingArguments(arguments)
                                        } else {
                                            error
                                        }
                                    });
                                }
                            }));
                    }
                }
                // Parse any remaining options.
                if !end_of_options {
                    let parsed_context = parse_context(
                        args,
                        &mut Shape::Empty {
                            description: String::new(),
                        },
                        &mut combined_options,
                        context,
                    );
                    context = parsed_context.context?;
                    'outer: for (optional_name, optional_context) in parsed_context.options {
                        // Find whether the optional name is in this struct.
                        for optional_field in &mut *optional {
                            if optional_name == optional_field.name
                                || optional_field.aliases.contains(&optional_name)
                            {
                                context.segments.push(Segment::Context(optional_context));
                                continue 'outer;
                            }
                        }
                        parsed_options.push((optional_name, optional_context));
                    }
                    if parsed_context.closing_end_of_options {
                        closing_end_of_options = true;
                    }
                }
            }
            Shape::Enum { name, variants, .. } => {
                // Parse the variant.
                'outer: loop {
                    let token = args
                        .next_token()
                        .ok_or(Error::MissingArguments(vec![name.to_owned()]))?;
                    match token {
                        Token::Positional(variant_name) => {
                            let variant_name_str = str::from_utf8(&variant_name).or_else(|_| {
                                Err(Error::UnrecognizedVariant {
                                    name: String::from_utf8_lossy(&variant_name).into(),
                                    expecting: variants
                                        .iter()
                                        .map(|variant| {
                                            iter::once(variant.name)
                                                .chain(variant.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                })
                            })?;
                            for variant in variants.clone() {
                                if let Some(static_variant_name) = iter::once(variant.name)
                                    .chain(variant.aliases)
                                    .find(|s| *s == variant_name_str)
                                {
                                    *shape = Shape::Variant {
                                        name: static_variant_name,
                                        shape: Box::new(variant.shape),
                                        description: variant.description,
                                        enum_name: name,
                                        variants: variants.clone(),
                                    };

                                    if let Shape::Variant {
                                        shape: inner_shape, ..
                                    } = shape
                                    {
                                        context
                                            .segments
                                            .push(Segment::Identifier(static_variant_name));
                                        // Parse the variant's shape.
                                        let parsed_context =
                                            parse_context(args, inner_shape, options, context);
                                        // Handle options.
                                        parsed_options.extend(parsed_context.options);
                                        if parsed_context.closing_end_of_options {
                                            closing_end_of_options = true;
                                        }
                                        context = parsed_context.context?;
                                    } else {
                                        unreachable!()
                                    }
                                    break 'outer;
                                }
                            }
                            return Err(Error::UnrecognizedVariant {
                                name: variant_name_str.into(),
                                expecting: variants
                                    .iter()
                                    .map(|variant| {
                                        iter::once(variant.name)
                                            .chain(variant.aliases.iter().copied())
                                    })
                                    .flatten()
                                    .collect(),
                            });
                        }
                        Token::Optional(value) => {
                            let identifier =
                                str::from_utf8(&value).or(Err(Error::UnrecognizedOption {
                                    name: String::from_utf8_lossy(&value).into(),
                                    expecting: options
                                        .iter()
                                        .map(|field| {
                                            iter::once(field.name)
                                                .chain(field.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                }))?;
                            let mut optional_context = Context { segments: vec![] };
                            let mut found = false;
                            let mut index = 0;
                            while index < options.len() {
                                let optional_field = &options[index];
                                if let Some(static_field_name) = iter::once(optional_field.name)
                                    .chain(optional_field.aliases.clone())
                                    .find(|s| *s == identifier)
                                {
                                    let mut optional_field = options.remove(index);
                                    found = true;
                                    optional_context
                                        .segments
                                        .push(Segment::Identifier(static_field_name));
                                    let parsed_context = parse_context(
                                        args,
                                        &mut optional_field.shape,
                                        options,
                                        optional_context,
                                    );
                                    parsed_options.extend(parsed_context.options);
                                    parsed_options
                                        .push((static_field_name, parsed_context.context?));
                                    if parsed_context.closing_end_of_options {
                                        closing_end_of_options = true;
                                    }
                                    options.insert(index, optional_field);
                                    break;
                                } else {
                                    index += 1;
                                }
                            }
                            if !found {
                                return Err(Error::UnrecognizedOption {
                                    name: identifier.into(),
                                    expecting: options
                                        .iter()
                                        .map(|field| {
                                            iter::once(field.name)
                                                .chain(field.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                });
                            }
                        }
                        Token::EndOfOptions => {
                            let variant_name = args
                                .next_positional()
                                .ok_or(Error::MissingArguments(vec![name.to_owned()]))?;
                            let variant_name_str = str::from_utf8(&variant_name).or_else(|_| {
                                Err(Error::UnrecognizedVariant {
                                    name: String::from_utf8_lossy(&variant_name).into(),
                                    expecting: variants
                                        .iter()
                                        .map(|variant| {
                                            iter::once(variant.name)
                                                .chain(variant.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                })
                            })?;
                            for variant in variants.clone() {
                                if let Some(static_variant_name) = iter::once(variant.name)
                                    .chain(variant.aliases)
                                    .find(|s| *s == variant_name_str)
                                {
                                    *shape = Shape::Variant {
                                        name: static_variant_name,
                                        shape: Box::new(variant.shape),
                                        description: variant.description,
                                        enum_name: name,
                                        variants: variants.clone(),
                                    };
                                    if let Shape::Variant {
                                        shape: inner_shape, ..
                                    } = shape
                                    {
                                        context
                                            .segments
                                            .push(Segment::Identifier(static_variant_name));
                                        context =
                                            parse_context_no_options(args, inner_shape, context)?;
                                    } else {
                                        unreachable!();
                                    }
                                    break 'outer;
                                }
                            }
                            return Err(Error::UnrecognizedVariant {
                                name: variant_name_str.into(),
                                expecting: variants
                                    .iter()
                                    .map(|variant| {
                                        iter::once(variant.name)
                                            .chain(variant.aliases.iter().copied())
                                    })
                                    .flatten()
                                    .collect(),
                            });
                        }
                    }
                }
            }
            Shape::Variant {
                enum_name,
                variants,
                ..
            } => {
                // Parse the variant.
                loop {
                    let token = args
                        .next_token()
                        .ok_or(Error::MissingArguments(vec![enum_name.to_owned()]))?;
                    match token {
                        Token::Positional(variant_name) => {
                            let variant_name_str = str::from_utf8(&variant_name).or_else(|_| {
                                Err(Error::UnrecognizedVariant {
                                    name: String::from_utf8_lossy(&variant_name).into(),
                                    expecting: variants
                                        .iter()
                                        .map(|variant| {
                                            iter::once(variant.name)
                                                .chain(variant.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                })
                            })?;
                            let mut found = false;
                            for mut variant in variants.clone() {
                                if let Some(static_variant_name) = iter::once(variant.name)
                                    .chain(variant.aliases)
                                    .find(|s| *s == variant_name_str)
                                {
                                    context
                                        .segments
                                        .push(Segment::Identifier(static_variant_name));
                                    // Parse the variant's shape.
                                    let parsed_context =
                                        parse_context(args, &mut variant.shape, options, context);
                                    // Handle options.
                                    parsed_options.extend(parsed_context.options);
                                    if parsed_context.closing_end_of_options {
                                        closing_end_of_options = true;
                                    }
                                    context = parsed_context.context?;
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                return Err(Error::UnrecognizedVariant {
                                    name: variant_name_str.into(),
                                    expecting: variants
                                        .iter()
                                        .map(|variant| {
                                            iter::once(variant.name)
                                                .chain(variant.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                });
                            }
                            break;
                        }
                        Token::Optional(value) => {
                            let identifier =
                                str::from_utf8(&value).or(Err(Error::UnrecognizedOption {
                                    name: String::from_utf8_lossy(&value).into(),
                                    expecting: options
                                        .iter()
                                        .map(|field| {
                                            iter::once(field.name)
                                                .chain(field.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                }))?;
                            let mut optional_context = Context { segments: vec![] };
                            let mut found = false;
                            let mut index = 0;
                            while index < options.len() {
                                let optional_field = &options[index];
                                if let Some(static_field_name) = iter::once(optional_field.name)
                                    .chain(optional_field.aliases.clone())
                                    .find(|s| *s == identifier)
                                {
                                    let mut optional_field = options.remove(index);
                                    found = true;
                                    optional_context
                                        .segments
                                        .push(Segment::Identifier(static_field_name));
                                    let parsed_context = parse_context(
                                        args,
                                        &mut optional_field.shape,
                                        options,
                                        optional_context,
                                    );
                                    parsed_options.extend(parsed_context.options);
                                    parsed_options
                                        .push((static_field_name, parsed_context.context?));
                                    if parsed_context.closing_end_of_options {
                                        closing_end_of_options = true;
                                    }
                                    options.insert(index, optional_field);
                                    break;
                                } else {
                                    index += 1;
                                }
                            }
                            if !found {
                                return Err(Error::UnrecognizedOption {
                                    name: identifier.into(),
                                    expecting: options
                                        .iter()
                                        .map(|field| {
                                            iter::once(field.name)
                                                .chain(field.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                });
                            }
                        }
                        Token::EndOfOptions => {
                            let variant_name = args
                                .next_positional()
                                .ok_or(Error::MissingArguments(vec![enum_name.to_owned()]))?;
                            let variant_name_str = str::from_utf8(&variant_name).or_else(|_| {
                                Err(Error::UnrecognizedVariant {
                                    name: String::from_utf8_lossy(&variant_name).into(),
                                    expecting: variants
                                        .iter()
                                        .map(|variant| {
                                            iter::once(variant.name)
                                                .chain(variant.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                })
                            })?;
                            let mut found = false;
                            for mut variant in variants.clone() {
                                if let Some(static_variant_name) = iter::once(variant.name)
                                    .chain(variant.aliases)
                                    .find(|s| *s == variant_name_str)
                                {
                                    context
                                        .segments
                                        .push(Segment::Identifier(static_variant_name));
                                    context = parse_context_no_options(
                                        args,
                                        &mut variant.shape,
                                        context,
                                    )?;
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                return Err(Error::UnrecognizedVariant {
                                    name: variant_name_str.into(),
                                    expecting: variants
                                        .iter()
                                        .map(|variant| {
                                            iter::once(variant.name)
                                                .chain(variant.aliases.iter().copied())
                                        })
                                        .flatten()
                                        .collect(),
                                });
                            }
                            break;
                        }
                    }
                }
            }
        }
        Ok(context)
    })();

    ParsedContext {
        context,
        options: parsed_options,
        closing_end_of_options,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, Context, Error, Segment};
    use crate::trace::{Field, Shape};
    use claims::{assert_err_eq, assert_ok_eq};

    #[test]
    fn parse_empty() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Empty {
                    description: String::new()
                },
            ),
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
                    name: "bar".to_owned(),
                    description: String::new(),
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
                vec!["--"],
                &mut Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                }
            ),
            Error::MissingArguments(vec!["bar".to_owned()])
        );
    }

    #[test]
    fn parse_optional_primitive() {
        assert_ok_eq!(
            parse(
                ["--foo"],
                &mut Shape::Optional(Box::new(Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
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
                    name: "",
                    description: String::new(),
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
                    name: "",
                    description: String::new(),
                    required: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                        }
                    }],
                    optional: vec![],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("bar"), Segment::Value("foo".into())],
                }),]
            }
        );
    }

    #[test]
    fn parse_struct_multiple_fields() {
        assert_ok_eq!(
            parse(
                vec!["foo", "bar"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned(),
                                description: String::new(),
                            }
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned(),
                                description: String::new(),
                            }
                        }
                    ],
                    optional: vec![],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("baz"), Segment::Value("foo".into())],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("qux"), Segment::Value("bar".into())],
                    }),
                ]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
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
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
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
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec!["qux"],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
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
        assert_ok_eq!(
            parse(
                vec!["--qux", "foo", "--bar", "baz"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec!["qux"],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                        },
                    }],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("qux"), Segment::Value("foo".into())]
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("baz".into())]
                    })
                ]
            },
        );
    }

    #[test]
    fn parse_struct_mixed_fields() {
        assert_ok_eq!(
            parse(
                vec!["123", "--bar", "foo", "456", "--qux", "789"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "quux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "missing",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            }
                        },
                    ]
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("foo"), Segment::Value("123".into())],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("foo".into())],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("quux"), Segment::Value("456".into())],
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
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "inner_struct",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Struct {
                                name: "",
                                description: String::new(),
                                required: vec![Field {
                                    name: "foo",
                                    description: String::new(),
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                    },
                                },],
                                optional: vec![Field {
                                    name: "bar",
                                    description: String::new(),
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                    },
                                },],
                            }
                        },
                        Field {
                            name: "quux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "missing",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("qux"), Segment::Value("789".into()),],
                    }),
                    Segment::Context(Context {
                        segments: vec![
                            Segment::Identifier("inner_struct"),
                            Segment::Context(Context {
                                segments: vec![
                                    Segment::Identifier("foo"),
                                    Segment::Value("123".into())
                                ],
                            }),
                            Segment::Context(Context {
                                segments: vec![
                                    Segment::Identifier("bar"),
                                    Segment::Value("foo".into()),
                                ]
                            }),
                        ],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("quux"), Segment::Value("456".into())],
                    }),
                ]
            }
        );
    }
}
