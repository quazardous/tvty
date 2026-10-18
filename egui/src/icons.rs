//! tvty's icons: its own (the app's) and the Material Symbols the GPUI
//! build draws (`assets/icons/`), registered with egui-pop's icon set,
//! once, before the theme is installed (their textures are made then).

use std::sync::OnceLock;

use egui_pop::core::icon::Icon;
use egui_pop::core::icon_set::{self, Paint};

macro_rules! icons {
    ($($name:ident => $file:literal, $paint:ident;)*) => {
        $(
            pub fn $name() -> Option<Icon> {
                static ICON: OnceLock<Option<Icon>> = OnceLock::new();
                *ICON.get_or_init(|| {
                    icon_set::register(
                        concat!("tvty.", stringify!($name)),
                        &include_bytes!(concat!("../../assets/", $file))[..],
                        Paint::$paint,
                    )
                    .ok()
                })
            }
        )*

        /// Registers every icon.
        pub fn register() {
            $(let _ = $name();)*
        }
    };
}

icons! {
    app => "tvty.svg", Colour;
    pinned => "icons/keep_fill.svg", Mono;
    megaphone => "icons/megaphone.svg", Mono;
    message_all => "icons/bot-message-square.svg", Mono;
}
