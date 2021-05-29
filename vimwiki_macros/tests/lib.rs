// Provide crate rename at top level as macros should never be used directly
// and therefore we are expecting ::vimwiki or similar
extern crate vimwiki_core as vimwiki;

mod functions;
mod hygiene;
