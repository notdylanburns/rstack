pub mod common;
pub mod protocols;

#[macro_use]
pub(crate) mod util {
    macro_rules! getter {
        ($ident:ident: $t:ty) => {
            pub fn $ident(&self) -> $t {
                self.$ident
            }
        };
    }

    pub(super) use getter;
}