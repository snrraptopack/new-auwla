use crate::emitter::JsEmitter;
use auwla_ast::{Attribute, Method, Type};

impl JsEmitter {
    /// Emit the body of an `@external(js, ...)` method into the extensions output.
    ///
    /// Handles all mapping types (property, method, static, constructor) and
    /// automatically wraps the return value in Optional `{ ok, value }` when
    /// the method's return type is `Optional<T>`.
    ///
    /// Returns `true` if the attribute was handled, `false` otherwise.
    pub(crate) fn emit_external_body(
        &mut self,
        attr: &Attribute,
        method: &Method,
        wrap_optional: bool,
    ) -> bool {
        if attr.args.first().map(|s| s.as_str()) != Some("js") {
            return false;
        }

        let mapping_type = attr.args.get(1).map(|s| s.as_str());
        let needs_optional_wrap =
            wrap_optional && matches!(&method.return_ty, Some(Type::Optional(_)));

        match mapping_type {
            Some("property") => {
                let target = attr
                    .args
                    .get(2)
                    .map(|s| s.as_str())
                    .expect("Missing JS property name in @external attribute");
                let call = format!("__self.{}", target);
                self.emit_external_return(&call, needs_optional_wrap);
            }
            Some("method") => {
                let target = attr
                    .args
                    .get(2)
                    .map(|s| s.as_str())
                    .expect("Missing JS method name in @external attribute");
                let args = Self::non_self_param_names(method);
                let call = format!("__self.{}({})", target, args.join(", "));
                self.emit_external_return(&call, needs_optional_wrap);
            }
            Some("static") => {
                let obj = attr
                    .args
                    .get(2)
                    .map(|s| s.as_str())
                    .expect("Missing JS object name in @external static attribute");
                let target = attr
                    .args
                    .get(3)
                    .map(|s| s.as_str())
                    .expect("Missing JS static member name in @external attribute");
                let args: Vec<&str> = method.params.iter().map(|(n, _)| n.as_str()).collect();
                let call = format!("{}.{}({})", obj, target, args.join(", "));
                self.emit_external_return(&call, needs_optional_wrap);
            }
            _ => return false,
        }

        true
    }

    /// Emit a return statement for an external call, optionally wrapping
    /// in the Optional `{ ok, value }` shape.
    fn emit_external_return(&mut self, call: &str, wrap_optional: bool) {
        if wrap_optional {
            self.write_indent_ext();
            self.write_ext(&format!("const _res = {};\n", call));
            self.write_indent_ext();
            self.write_ext("return (_res != null) ? { ok: true, value: _res } : { ok: false };\n");
        } else {
            self.write_indent_ext();
            self.write_ext(&format!("return {};\n", call));
        }
    }

    /// Collect parameter names excluding `self`.
    fn non_self_param_names(method: &Method) -> Vec<&str> {
        method
            .params
            .iter()
            .filter(|(n, _)| n != "self")
            .map(|(n, _)| n.as_str())
            .collect()
    }
}
