mod descriptions;

pub(crate) use descriptions::{descriptions, Descriptions};

use crate::Container;
use syn::{Ident, Visibility};

pub(crate) fn identifier(container: &Container) -> &Ident {
    match container {
        Container::Enum(item) => &item.ident,
        Container::Struct(item) => &item.ident,
    }
}

pub(crate) fn visibility(container: &Container) -> &Visibility {
    match container {
        Container::Enum(item) => &item.vis,
        Container::Struct(item) => &item.vis,
    }
}
