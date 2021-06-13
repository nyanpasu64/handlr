use crate::common::{Handler, MimeOrExtension};

#[derive(clap::Clap)]
#[clap(global_setting = clap::AppSettings::DeriveDisplayOrder)]
#[clap(global_setting = clap::AppSettings::DisableHelpSubcommand)]
#[clap(version = clap::crate_version!())]
pub enum Cmd {
    /// List default apps and the associated handlers
    List {
        #[clap(long, short)]
        all: bool,
    },

    /// Set the default handler for mime/extension
    Set {
        mime: MimeOrExtension,
        handler: Handler,
    },

    /// Unset the default handler for mime/extension
    Unset { mime: MimeOrExtension },

    /// Add a handler for given mime/extension
    /// Note that the first handler is the default
    Add {
        mime: MimeOrExtension,
        handler: Handler,
    },
}
