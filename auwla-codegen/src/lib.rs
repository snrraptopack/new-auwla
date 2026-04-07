mod emitter;
mod external;
mod writer;

pub use emitter::emit_js;

pub mod expr;
pub mod match_logic; // renamed to match_logic because match is a keyword
pub mod pattern;
pub mod postprocess;
pub mod stmt;
pub mod try_logic; // renamed to try_logic for consistency

#[cfg(test)]
mod tests {
	use std::collections::{HashMap, HashSet};

	use auwla_ast::{
		Attribute, ExtensionOrigin, Method, Program, Spanned, StmtKind, Type,
	};

	use crate::emit_js;

	#[test]
	fn malformed_external_attribute_does_not_panic_codegen() {
		let method = Method {
			name: "broken_ext".to_string(),
			attributes: vec![Attribute {
				name: "external".to_string(),
				args: vec!["native".to_string(), "method".to_string()],
			}],
			params: vec![("self".to_string(), None, false)],
			return_ty: Some(Type::Basic("number".to_string())),
			body: vec![],
			is_static: false,
			type_params: None,
			span: 0..0,
			operator: None,
		};

		let program = Program {
			statements: vec![Spanned::new(
				StmtKind::Extend {
					type_params: None,
					target_type: Type::Basic("number".to_string()),
					methods: vec![method],
				},
				0..0,
			)],
		};

		let result = std::panic::catch_unwind(|| {
			emit_js(
				&program,
				&HashMap::new(),
				&HashSet::new(),
				&HashMap::new(),
				&HashMap::new(),
				ExtensionOrigin::User,
			)
		});

		assert!(result.is_ok(), "codegen should not panic on malformed @external args");
	}
}
