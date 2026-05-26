//! AI projection view for SDIF documents.
//!
//! Produces compact `.sdif.ai` output from a parsed [`crate::Document`].

use crate::Document;

/// Return the AI projection of `doc` as valid `.sdif.ai` text.
///
/// The projection keeps document content intact and swaps the format directive
/// to `@sdif.ai 1.0`, then delegates ordering/formatting to the canonical
/// serializer.
pub fn ai_view(doc: &Document) -> String {
    let mut ai_doc = doc.clone();
    for directive in &mut ai_doc.directives {
        if directive.name == "sdif" || directive.name == "sdif.ai" {
            directive.name = "sdif.ai".to_owned();
        }
    }
    crate::canonicalize(&ai_doc)
}
