use crate::emitter::JsEmitter;
use auwla_ast::expr::Expr;

impl JsEmitter {
    /// Emit a nested try expression as an IIFE.
    pub(crate) fn emit_try_expr(&mut self, expr: &Expr, error_expr: &Option<Box<Expr>>) {
        self.write("(() => { ");
        let temp = self.fresh_temp();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(expr);
        self.write(&format!("; if (!{}.ok) throw new Error(", temp));
        if let Some(err) = error_expr {
            self.emit_expr(err);
        } else {
            self.write(&format!("{}.value", temp));
        }
        self.write("); return ");
        self.write(&temp);
        self.write(".value; })()");
    }

    /// Emit a top-level try assignment: `const name = tried?("err");`
    pub(crate) fn emit_try_assign(
        &mut self,
        decl_kw: &str,
        name: &str,
        tried: &Expr,
        error_expr: &Option<Box<Expr>>,
    ) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(tried);
        self.write(";\n");

        self.write_indent();
        self.write(&format!("if (!{}.ok) throw new Error(", temp));
        if let Some(err) = error_expr {
            self.emit_expr(err);
        } else {
            self.write(&format!("{}.value", temp));
        }
        self.write(");\n");

        self.write_indent();
        if !decl_kw.is_empty() {
            self.write(&format!("{} {} = {}.value;\n", decl_kw, name, temp));
        } else {
            self.write(&format!("{} = {}.value;\n", name, temp));
        }
    }

    /// Emit a standalone try statement: `tried?;`
    pub(crate) fn emit_try_standalone(&mut self, tried: &Expr, error_expr: &Option<Box<Expr>>) {
        let temp = self.fresh_temp();
        self.write_indent();
        self.write(&format!("const {} = ", temp));
        self.emit_expr(tried);
        self.write(";\n");

        self.write_indent();
        self.write(&format!("if (!{}.ok) throw new Error(", temp));
        if let Some(err) = error_expr {
            self.emit_expr(err);
        } else {
            self.write(&format!("{}.value", temp));
        }
        self.write(");\n");
    }
}
