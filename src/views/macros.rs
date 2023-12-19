#[macro_export]
macro_rules! inner_generate_by_params {
    ($key:ident, $key_display:expr, $default:expr) => {
        paste::paste! {
            fn [<inner_get_page_ $key>](query: &Value) -> anyhow::Result<u64> {
                let value = query.get(Self::[<page_ $key _param>]());
                Ok(value.context($crate::error::OptionValueNoneSnafu)?.as_str().unwrap_or("").parse::<u64>()?)
            }

            #[inline]
            fn [<page_ $key _param>]() -> &'static str {
                concat!("page_", $key_display)
            }

            #[inline]
            fn [<default_page_ $key>]() -> u64 {
                $default
            }
        }
    };
}

#[macro_export]
macro_rules! generate_by_params {
       ($key:ident, $key_display:expr, $default:expr) => {
        paste::paste! {
            fn [<get_page_ $key>](query: &Value) -> u64 {
                let value = Self::[<inner_get_page_ $key>](query).unwrap_or(Self::[<default_page_ $key>]());
                tracing::debug!("params for {} is {}", Self::[<page_ $key _param>](), value);
                value
            }
        }
        $crate::inner_generate_by_params!($key, $key_display, $default);
    };
    ($key:ident, $key_display:expr, $default:expr, $reduce:expr) => {
        paste::paste! {
            fn [<get_page_ $key>](query: &Value) -> u64 {
                let mut value = Self::[<inner_get_page_ $key>](query).unwrap_or(Self::[<default_page_ $key>]());
                if value >= $reduce {
                    value -= $reduce;
                }
                tracing::debug!("params for {} is {}", Self::[<page_ $key _param>](), value);
                value
            }
        }
        $crate::inner_generate_by_params!($key, $key_display, $default);
    };
}
