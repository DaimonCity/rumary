use crate::domain::api::LoaderError;
use crate::domain::name::{DescriptionError, DirectoryNameError, DisplayNameError};
use crate::domain::url::IconUrlError;
use crate::domain::user::{HashError, LoginError, NicknameError};
use crate::domain::version::VersionError;

#[macro_export]
macro_rules! err_from {
    ($from:ty, $to:ty, $enum_name:ident) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                Self::$enum_name(value)
            }
        }
    };
}

pub enum ValueObjectError {
    DirectoryName(DirectoryNameError),
    DisplayName(DisplayNameError),
    Description(DescriptionError),
    IconUrl(IconUrlError),
    Nickname(NicknameError),
    LoaderError(LoaderError),
    Login(LoginError),
    PasswordHash(HashError),
    Version(VersionError),
}

err_from!(DirectoryNameError, ValueObjectError, DirectoryName);
err_from!(DisplayNameError, ValueObjectError, DisplayName);
err_from!(DescriptionError, ValueObjectError, Description);
err_from!(IconUrlError, ValueObjectError, IconUrl);
err_from!(NicknameError, ValueObjectError, Nickname);
err_from!(LoaderError, ValueObjectError, LoaderError);
err_from!(LoginError, ValueObjectError, Login);
err_from!(HashError, ValueObjectError, PasswordHash);
err_from!(VersionError, ValueObjectError, Version);

#[macro_export]
macro_rules! impl_new {
    ($name:ty, $( $arg:ident : $type:ty ),* , $code:block) => {
        impl $name {
            fn new( $( $arg : $type ),* ) -> Self {
                $code
            }
        }
    };
    ($name:ty, $( $arg:ident : $type:ty ),*) => {
        impl $name {
            pub fn new( $( $arg : $type ),* ) -> Self {
                Self {
                    $($arg),*
                }
            }
        }
    };
}
