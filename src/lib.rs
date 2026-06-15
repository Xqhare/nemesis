#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::all)]
#![warn(clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::print_stdout,
    clippy::implicit_return,
    clippy::single_call_fn,
    clippy::str_to_string,
    clippy::question_mark_used,
    clippy::indexing_slicing,
    clippy::pattern_type_mismatch,
    clippy::arbitrary_source_item_ordering,
    clippy::doc_paragraphs_missing_punctuation,
    clippy::exhaustive_enums,
    clippy::min_ident_chars,
    clippy::missing_trait_methods,
    clippy::impl_trait_in_params,
    clippy::as_conversions,
    clippy::cast_lossless,
    clippy::shadow_reuse,
    clippy::blanket_clippy_restriction_lints,
    clippy::doc_include_without_cfg,
    clippy::missing_inline_in_public_items,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::iter_without_into_iter,
    clippy::std_instead_of_core,
    clippy::absolute_paths,
    clippy::allow_attributes_without_reason,
    clippy::ref_patterns,
    clippy::single_char_lifetime_names,
    clippy::pub_use,
    clippy::std_instead_of_alloc,
    clippy::return_self_not_must_use,
    clippy::unreachable,
    clippy::arithmetic_side_effects,
    clippy::uninlined_format_args
)]

mod error;
pub use error::{
    NemesisChainIter, NemesisCollection, NemesisError, NemesisPayload, NemesisResultExt,
};
