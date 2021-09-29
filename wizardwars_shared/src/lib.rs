pub mod components;
pub mod events;
pub mod messages;
pub mod network;
pub mod systems;

#[macro_export]
macro_rules! enum_from {
    ($enum:ident, $variant:ident) => {
        impl From<$value> for $enum {
            fn from(val: $variant) -> Self {
                Self::$variant(val)
            }
        }
    };
    ($enum:ident, $variant:ident, $value:ident) => {
        impl From<$value> for $enum {
            fn from(val: $value) -> Self {
                Self::$variant(val)
            }
        }
    };
}
