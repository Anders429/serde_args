mod context;
mod error;
mod token;

pub(crate) use context::{
    Context,
    ContextIter,
    Segment,
};
pub(crate) use error::Error;

use crate::trace::{
    Field,
    Shape,
};
use std::{
    ffi::OsString,
    iter,
    str,
    vec,
};
use token::{
    ParsedArgs,
    Token,
};

pub(crate) fn parse<Arg, Args>(args: Args, shape: &mut Shape) -> Result<Context, Error>
where
    Args: IntoIterator<Item = Arg>,
    Arg: Into<OsString>,
{
    let mut parsed_args = ParsedArgs::new(args.into_iter().map(|arg| arg.into()));
    let mut override_options = vec![Field {
        name: "help",
        description: "Display this message.".into(),
        aliases: vec!["h"],
        shape: Shape::Empty {
            description: String::new(),
        },
        index: 0,
    }];
    let parsed_context = parse_context(
        &mut parsed_args,
        shape,
        &mut override_options,
        Context { segments: vec![] },
    );

    // Parse any remaining options that are at the end of the tokens.
    // This should just include things like `--help`.
    let options = if parsed_context.closing_end_of_options {
        parsed_context.options
    } else {
        let closing_parsed_context = parse_context(
            &mut parsed_args,
            &mut Shape::Empty {
                description: String::new(),
            },
            &mut override_options,
            Context { segments: vec![] },
        );
        let _ = closing_parsed_context.context?;
        let mut options = parsed_context.options;
        options.extend(closing_parsed_context.options);
        options
    };

    // Handle overriding options.
    for (option_name, _option_context) in options {
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
        } else if let Some(token) = parsed_args.next_token() {
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
        Shape::Primitive { ref name, .. } | Shape::Boolean { ref name, .. } => {
            context.segments.push(Segment::Value(
                args.next_positional()
                    .ok_or(Error::MissingArguments(vec![name.clone()]))?,
            ));
            Ok(context)
        }
        Shape::Optional(ref mut optional_shape) => {
            // This is a "positional optional". It starts its own isolated context, which only
            // contains its own optional value if it exists.
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
                Shape::Primitive { .. } | Shape::Boolean { .. } | Shape::Enum { .. } => {
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
                Shape::Struct { ref required, .. } => {
                    if let Some(optional) = args.next_optional() {
                        // If the value we extracted is empty, we only revisit it if there is at
                        // least one required field that is not an empty field.
                        if !optional.is_empty()
                            || required
                                .iter()
                                .any(|field| !matches!(field.shape, Shape::Empty { .. }))
                        {
                            args.revisit = Some(optional);
                        }
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
            ref mut booleans,
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
                                            required_iter
                                                .filter(|field| {
                                                    !matches!(field.shape, Shape::Empty { .. })
                                                })
                                                .map(|field| field.name.to_owned()),
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
                        &mut optional
                            .clone()
                            .into_iter()
                            .chain(booleans.clone())
                            .collect(),
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
                                            required_iter
                                                .filter(|field| {
                                                    !matches!(field.shape, Shape::Empty { .. })
                                                })
                                                .map(|field| field.name.to_owned()),
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
                    for (optional_name, mut optional_context) in parsed_options {
                        let mut found = false;
                        // Find whether the optional name is in this struct.
                        for optional_field in (&mut *optional).into_iter() {
                            if optional_name == optional_field.name
                                || optional_field.aliases.contains(&optional_name)
                            {
                                found = true;
                                context
                                    .segments
                                    .push(Segment::Context(optional_context.clone()));
                                break;
                            }
                        }
                        if !found {
                            for boolean_field in (&mut *booleans).into_iter() {
                                if optional_name == boolean_field.name
                                    || boolean_field.aliases.contains(&optional_name)
                                {
                                    found = true;
                                    optional_context
                                        .segments
                                        .push(Segment::Value(b"true".into()));
                                    context.segments.push(Segment::Context(optional_context));
                                    break;
                                }
                            }
                        }
                        if !found {
                            return Err(Error::UnrecognizedOption {
                                name: optional_name.into(),
                                expecting: optional
                                    .iter()
                                    .chain(booleans.iter())
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
                    &mut optional
                        .clone()
                        .into_iter()
                        .chain(booleans.clone())
                        .collect(),
                    context,
                );
                context = parsed_context.context?;
                for (optional_name, mut optional_context) in parsed_context.options {
                    let mut found = false;
                    // Find whether the optional name is in this struct.
                    for optional_field in &mut *optional {
                        if optional_name == optional_field.name
                            || optional_field.aliases.contains(&optional_name)
                        {
                            found = true;
                            context
                                .segments
                                .push(Segment::Context(optional_context.clone()));
                            break;
                        }
                    }
                    if !found {
                        for boolean_field in (&mut *booleans).into_iter() {
                            if optional_name == boolean_field.name
                                || boolean_field.aliases.contains(&optional_name)
                            {
                                found = true;
                                optional_context
                                    .segments
                                    .push(Segment::Value(b"true".into()));
                                context.segments.push(Segment::Context(optional_context));
                                break;
                            }
                        }
                    }
                    if !found {
                        return Err(Error::UnrecognizedOption {
                            name: optional_name.into(),
                            expecting: optional
                                .iter()
                                .chain(booleans.iter())
                                .map(|field| {
                                    iter::once(field.name).chain(field.aliases.iter().copied())
                                })
                                .flatten()
                                .collect(),
                        });
                    }
                }
            }
            // Fill in any missing boolean fields with false.
            let cloned_segments = context.segments.clone();
            let found_fields: Vec<_> = cloned_segments
                .iter()
                .filter_map(|segment| {
                    if let Segment::Context(field_context) = segment {
                        if let Some(Segment::Identifier(name)) = field_context.segments.first() {
                            Some(name)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            for boolean_field in booleans {
                // Check whether the field name or any aliases have been found.
                let mut found = false;
                for field_name in
                    iter::once(&boolean_field.name).chain(boolean_field.aliases.iter())
                {
                    if found_fields.contains(&&field_name) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    context.segments.push(Segment::Context(Context {
                        segments: vec![
                            Segment::Identifier(boolean_field.name),
                            Segment::Value(b"false".into()),
                        ],
                    }));
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
                                if identifier.chars().count() <= 1 {
                                    args.revisit = Some({
                                        let mut bytes = vec![b'-'];
                                        bytes.extend(value);
                                        bytes
                                    });
                                } else {
                                    args.revisit = Some({
                                        let mut bytes = vec![b'-', b'-'];
                                        bytes.extend(value);
                                        bytes
                                    });
                                }
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
            Shape::Primitive { ref name, .. } | Shape::Boolean { ref name, .. } => loop {
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
                // This is a "positional optional". It starts its own isolated context, which only
                // contains its own optional value if it exists.
                //
                // We therefore simply parse in a no-option context, thereby ignoring all parent
                // context options.
                context = parse_context_no_options(args, shape, context)?;
            }
            Shape::Struct {
                required,
                optional,
                booleans,
                ..
            } => {
                // Parse the struct in its own nested context.
                let mut end_of_options = false;
                let mut combined_options = options.clone();
                combined_options.extend(optional.clone());
                combined_options.extend(booleans.clone());
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
                                                required_iter
                                                    .filter(|field| {
                                                        !matches!(field.shape, Shape::Empty { .. })
                                                    })
                                                    .map(|field| field.name.to_owned()),
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
                        'outer: for (optional_name, mut optional_context) in found_parsed_options {
                            // Find whether the optional name is in this struct.
                            for optional_field in &mut *optional {
                                if optional_name == optional_field.name
                                    || optional_field.aliases.contains(&optional_name)
                                {
                                    context
                                        .segments
                                        .push(Segment::Context(optional_context.clone()));
                                    continue 'outer;
                                }
                            }
                            for boolean_field in &mut *booleans {
                                if optional_name == boolean_field.name
                                    || boolean_field.aliases.contains(&optional_name)
                                {
                                    optional_context
                                        .segments
                                        .push(Segment::Value(b"true".into()));
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
                                                required_iter
                                                    .filter(|field| {
                                                        !matches!(field.shape, Shape::Empty { .. })
                                                    })
                                                    .map(|field| field.name.to_owned()),
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
                    'outer: for (optional_name, mut optional_context) in parsed_context.options {
                        // Find whether the optional name is in this struct.
                        for optional_field in &mut *optional {
                            if optional_name == optional_field.name
                                || optional_field.aliases.contains(&optional_name)
                            {
                                context
                                    .segments
                                    .push(Segment::Context(optional_context.clone()));
                                continue 'outer;
                            }
                        }
                        for boolean_field in &mut *booleans {
                            if optional_name == boolean_field.name
                                || boolean_field.aliases.contains(&optional_name)
                            {
                                optional_context
                                    .segments
                                    .push(Segment::Value(b"true".into()));
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
                // Fill in any missing boolean fields with false.
                let cloned_segments = context.segments.clone();
                let found_fields: Vec<_> = cloned_segments
                    .iter()
                    .filter_map(|segment| {
                        if let Segment::Context(field_context) = segment {
                            if let Some(Segment::Identifier(name)) = field_context.segments.first()
                            {
                                Some(name)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                for boolean_field in booleans {
                    // Check whether the field name or any aliases have been found.
                    let mut found = false;
                    for field_name in
                        iter::once(&boolean_field.name).chain(boolean_field.aliases.iter())
                    {
                        if found_fields.contains(&&field_name) {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        context.segments.push(Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier(boolean_field.name),
                                Segment::Value(b"false".into()),
                            ],
                        }));
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
    use super::{
        parse,
        Context,
        Error,
        Segment,
    };
    use crate::trace::{
        Field,
        Shape,
        Variant,
    };
    use claims::{
        assert_err_eq,
        assert_ok_eq,
    };

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
    fn parse_primitive_no_args() {
        assert_err_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                }
            ),
            // No arguments at all when arguments are expected should trigger help.
            Error::Help
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
    fn parse_primitive_after_end_of_args() {
        assert_ok_eq!(
            parse(
                vec!["--", "foo"],
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
    fn parse_boolean() {
        assert_ok_eq!(
            parse(
                ["false"],
                &mut Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }
            ),
            Context {
                segments: vec![Segment::Value("false".into())],
            }
        );
    }

    #[test]
    fn parse_boolean_no_args() {
        assert_err_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }
            ),
            // No arguments at all when arguments are expected should trigger help.
            Error::Help
        );
    }

    #[test]
    fn parse_boolean_end_of_args() {
        assert_err_eq!(
            parse(
                vec!["--"],
                &mut Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }
            ),
            Error::MissingArguments(vec!["bar".to_owned()])
        );
    }

    #[test]
    fn parse_boolean_after_end_of_args() {
        assert_ok_eq!(
            parse(
                vec!["--", "false"],
                &mut Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }
            ),
            Context {
                segments: vec![Segment::Value("false".into())],
            }
        );
    }

    #[test]
    fn parse_optional_empty() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Empty {
                    description: String::new(),
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context { segments: vec![] })],
            }
        );
    }

    #[test]
    fn parse_optional_empty_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Optional(Box::new(Shape::Empty {
                    description: String::new(),
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_empty_end_of_options() {
        assert_ok_eq!(
            parse(
                ["--"],
                &mut Shape::Optional(Box::new(Shape::Empty {
                    description: String::new(),
                }))
            ),
            Context { segments: vec![] }
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
    fn parse_optional_primitive_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Optional(Box::new(Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_primitive_empty_value() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("".into())]
                })],
            }
        );
    }

    #[test]
    fn parse_optional_primitive_end_of_options() {
        assert_ok_eq!(
            parse(
                ["--"],
                &mut Shape::Optional(Box::new(Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_boolean() {
        assert_ok_eq!(
            parse(
                ["--false"],
                &mut Shape::Optional(Box::new(Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("false".into())]
                })],
            }
        );
    }

    #[test]
    fn parse_optional_boolean_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Optional(Box::new(Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_boolean_empty_value() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("".into())]
                })],
            }
        );
    }

    #[test]
    fn parse_optional_boolean_end_of_options() {
        assert_ok_eq!(
            parse(
                ["--"],
                &mut Shape::Optional(Box::new(Shape::Boolean {
                    name: "bar".to_owned(),
                    description: String::new(),
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_struct_empty() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context { segments: vec![] })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_end_of_options() {
        assert_ok_eq!(
            parse(
                ["--"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_struct_empty_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_struct_single_field() {
        assert_ok_eq!(
            parse(
                ["--foo"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                        },
                        index: 0,
                    }],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("foo".into())]
                    })],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_single_field_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                        },
                        index: 0,
                    }],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_struct_single_field_empty_string() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                        },
                        index: 0,
                    }],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("".into())]
                    })],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_single_field_empty_string_first_required_field_empty() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![
                        Segment::Context(Context {
                            segments: vec![Segment::Identifier("baz")],
                        }),
                        Segment::Context(Context {
                            segments: vec![Segment::Identifier("bar"), Segment::Value("".into())]
                        })
                    ],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_all_required_fields_empty() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            },
                            index: 1,
                        },
                    ],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![
                        Segment::Context(Context {
                            segments: vec![Segment::Identifier("bar")]
                        }),
                        Segment::Context(Context {
                            segments: vec![Segment::Identifier("baz")]
                        })
                    ],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_multiple_fields() {
        assert_ok_eq!(
            parse(
                ["--foo", "123"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                    optional: vec![],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("bar"),
                                Segment::Value("foo".into())
                            ]
                        }),
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("qux"),
                                Segment::Value("123".into())
                            ]
                        })
                    ],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_required_and_optional_fields() {
        assert_ok_eq!(
            parse(
                ["--foo", "123", "--baz", "quux"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                    optional: vec![Field {
                        name: "baz",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "string".to_owned(),
                            description: String::new(),
                        },
                        index: 2,
                    }],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("bar"),
                                Segment::Value("foo".into())
                            ]
                        }),
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("qux"),
                                Segment::Value("123".into())
                            ]
                        }),
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("baz"),
                                Segment::Value("quux".into())
                            ]
                        })
                    ],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_optional_fields_from_outer_context_not_allowed() {
        assert_err_eq!(
            parse(
                ["--foo", "--help", "123", "--baz", "quux"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                    optional: vec![Field {
                        name: "baz",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "string".to_owned(),
                            description: String::new(),
                        },
                        index: 2,
                    }],
                    booleans: vec![],
                }))
            ),
            Error::UnrecognizedOption {
                name: "help".into(),
                expecting: vec!["baz".into()]
            }
        );
    }

    #[test]
    fn parse_optional_struct_only_optional_fields() {
        assert_ok_eq!(
            parse(
                ["-", "--baz", "quux"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                    booleans: vec![],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Context(Context {
                        segments: vec![Segment::Identifier("baz"), Segment::Value("quux".into())]
                    })],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_struct_only_boolean_fields() {
        assert_ok_eq!(
            parse(
                ["-", "--baz"],
                &mut Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![
                        Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("baz"),
                                Segment::Value("true".into())
                            ]
                        }),
                        Segment::Context(Context {
                            segments: vec![
                                Segment::Identifier("bar"),
                                Segment::Value("false".into())
                            ]
                        })
                    ],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_enum() {
        assert_ok_eq!(
            parse(
                ["--foo"],
                &mut Shape::Optional(Box::new(Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("foo")],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_enum_short() {
        assert_ok_eq!(
            parse(
                ["-f"],
                &mut Shape::Optional(Box::new(Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec!["f"],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("f")],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_enum_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Optional(Box::new(Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }))
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_optional_enum_with_value() {
        assert_ok_eq!(
            parse(
                ["--foo", "bar"],
                &mut Shape::Optional(Box::new(Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "string".into(),
                            description: String::new(),
                        }
                    }],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("foo"), Segment::Value("bar".into())],
                })]
            }
        );
    }

    #[test]
    fn parse_optional_enum_empty_variant() {
        assert_ok_eq!(
            parse(
                ["-"],
                &mut Shape::Optional(Box::new(Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("")],
                })]
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
                    booleans: vec![],
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
                        },
                        index: 0,
                    }],
                    optional: vec![],
                    booleans: vec![],
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
                            },
                            index: 0,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
                        }
                    ],
                    optional: vec![],
                    booleans: vec![],
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
                        index: 0,
                    }],
                    booleans: vec![],
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
                        index: 0,
                    }],
                    booleans: vec![],
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
                        index: 0,
                    }],
                    booleans: vec![],
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
                        index: 0,
                    }],
                    booleans: vec![],
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
    fn parse_struct_single_boolean_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        },
                        index: 0,
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("bar"), Segment::Value("false".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_boolean_present() {
        assert_ok_eq!(
            parse(
                vec!["--bar"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        },
                        index: 0,
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("bar"), Segment::Value("true".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_boolean_present_alias() {
        assert_ok_eq!(
            parse(
                vec!["--qux"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec!["qux"],
                        shape: Shape::Empty {
                            description: String::new(),
                        },
                        index: 0,
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("qux"), Segment::Value("true".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_boolean_present_multiple_aliases() {
        assert_ok_eq!(
            parse(
                vec!["--qux", "--bar"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec!["qux"],
                        shape: Shape::Empty {
                            description: String::new(),
                        },
                        index: 0,
                    }],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("qux"), Segment::Value("true".into())]
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("true".into())]
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
                            index: 0,
                        },
                        Field {
                            name: "quux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
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
                            index: 2,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 3,
                        },
                        Field {
                            name: "missing",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 4,
                        },
                    ],
                    booleans: vec![],
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
                                    index: 0,
                                },],
                                optional: vec![Field {
                                    name: "bar",
                                    description: String::new(),
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                    },
                                    index: 1,
                                },],
                                booleans: vec![],
                            },
                            index: 0,
                        },
                        Field {
                            name: "quux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
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
                            index: 2,
                        },
                        Field {
                            name: "missing",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 3,
                        },
                    ],
                    booleans: vec![],
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

    #[test]
    fn parse_struct_mixed_fields_end_of_options() {
        assert_ok_eq!(
            parse(
                vec!["123", "--bar", "foo", "--", "--qux"],
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
                            index: 0,
                        },
                        Field {
                            name: "quux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 1,
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
                            index: 2,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 3,
                        },
                        Field {
                            name: "missing",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 4,
                        },
                    ],
                    booleans: vec![],
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
                        segments: vec![Segment::Identifier("quux"), Segment::Value("--qux".into())],
                    }),
                ]
            }
        );
    }

    #[test]
    fn parse_struct_nested_end_of_options() {
        assert_ok_eq!(
            parse(
                vec!["--", "--qux", "123", "--bar", "foo"],
                &mut Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "quux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 0,
                        },
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
                                    index: 0,
                                },],
                                optional: vec![Field {
                                    name: "bar",
                                    description: String::new(),
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                    },
                                    index: 1,
                                },],
                                booleans: vec![],
                            },
                            index: 1,
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
                            index: 2,
                        },
                        Field {
                            name: "missing",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            },
                            index: 3,
                        },
                    ],
                    booleans: vec![],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("quux"), Segment::Value("--qux".into())],
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
                ]
            }
        );
    }

    #[test]
    fn parse_enum() {
        assert_ok_eq!(
            parse(
                ["foo"],
                &mut Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Identifier("foo")],
            }
        );
    }

    #[test]
    fn parse_enum_from_multiple_variants() {
        assert_ok_eq!(
            parse(
                ["bar"],
                &mut Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![
                        Variant {
                            name: "foo",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            }
                        },
                        Variant {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                            }
                        }
                    ],
                }
            ),
            Context {
                segments: vec![Segment::Identifier("bar")],
            }
        );
    }

    #[test]
    fn parse_enum_alias() {
        assert_ok_eq!(
            parse(
                ["f"],
                &mut Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec!["f"],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Identifier("f")],
            }
        );
    }

    #[test]
    fn parse_enum_with_value() {
        assert_ok_eq!(
            parse(
                ["foo", "bar"],
                &mut Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "string".into(),
                            description: String::new(),
                        }
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Identifier("foo"), Segment::Value("bar".into())],
            }
        );
    }

    #[test]
    fn parse_enum_after_end_of_options() {
        assert_ok_eq!(
            parse(
                ["--", "foo"],
                &mut Shape::Enum {
                    name: "Enum",
                    description: String::new(),
                    variants: vec![Variant {
                        name: "foo",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                        }
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Identifier("foo")],
            }
        );
    }
}
