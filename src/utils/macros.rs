#[macro_export]
macro_rules! payload_enum_helper {
    (
        $(#[$attribute:meta])*
        enum $name:ident {$(
            $(#[$variant_attribute:meta])*
            $variant:ident $({
                $($field:ident: $field_type:ty),* $(,)?
            })? = $discriminant_literal:literal | $discriminant_expr:expr,
        )+}
    ) => {
        $(#[$attribute])*
        pub enum $name {$(
            $(#[$variant_attribute])*
            $variant $({
                $($field: $field_type),*
            })?,
        )+}

        impl $name {
            pub fn serialize_fields<S: Serializer>(&self, message: &mut S::SerializeMap) -> Result<(), S::Error> {
                match self {
                    $($name::$variant $({ $($field),* })? => {
                        message.serialize_entry("t", &($discriminant_expr))?;
                        $(
                            $(message.serialize_entry(stringify!($field), $field)?;)*
                        )?
                    }),+
                }

                Ok(())
            }
        }

        paste! {
            $(#[$attribute])*
            #[derive(Deserialize, Serialize)]
            // Tag `t` from the word `type`
            #[serde(rename_all = "snake_case", tag = "t")]
            enum [<$name Helper>] {$(
                $(#[$variant_attribute])*
                #[serde(rename = $discriminant_literal)]
                $variant $({
                    $($field: $field_type),*
                })?,
            )+}

            impl From<[<$name Helper>]> for $name {
                fn from(value: [<$name Helper>]) -> Self {
                    match value {
                        $([<$name Helper>]::$variant $({ $($field),* })? => $name::$variant $({ $($field),* })?),+
                    }
                }
            }
        }
    };
}
