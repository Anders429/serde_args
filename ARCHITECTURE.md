# Architecture

`serde_args` is split into four parts: tracing, parsing, deserialization, and errors. Each of these is contained in its own module.

## Tracing

The `trace` module contains all code for determining the **shape** of a type. A type's shape, in the context of this crate, contains information about the deserialization methods called by that type's `Deserialize` implementation. This information is used both in parsing and in error displaying.

Tracing uses its own deserializer to identify how a type attempts to be deserialized. This deserialization is intended to fail, but the error message will, instead of returning an error about deserialization, return the shape traced. In compound type cases, such as `struct`s, the deserializer is run multiple times to obtain information about each field in turn.

## Parsing

The `parse` module uses the shape obtained during tracing to iterate over the provided argument iterator. With the information obtained during tracing, the parser is able to find positional values in the order given while also finding optional values at any valid point, even within nested shapes. Parsed values are organized into a segments within a **context**. This organizes the parsed values into a form that is easily deserialized into the actual type.

## Deserialization

The `de` module uses the parsed context to deserialize the arguments into a concrete value. It uses the same type that was traced in the tracing step, but the deserializer here actually provides values to the visitors using the values found in the context.

## Errors

The `error` module displays all possible errors during this process. Any error obtained in the above 3 steps is converted into the error type found here. Errors will also be paired with the traced shape (if tracing completed successfully) so that errors can have access to the traced information about the type.

Note that both the `--help` and `--version` flags will result in errors during parsing. These use the shape information to display the requested message.
