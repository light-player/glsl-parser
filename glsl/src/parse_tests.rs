#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, boxed::Box, string::String, vec, vec::Vec};

use crate::parsers::*;
use crate::syntax;

// Helper to convert &str to Span for testing
fn span(s: &str) -> Span {
  Span::new(s)
}

// Helper to extract the result from SpanResult for testing
fn extract_result<T>((remaining, result): (Span, T)) -> (&str, T) {
  (remaining.fragment(), result)
}

// Helper to normalize spans in AST nodes for comparison
// This allows tests to compare AST structure without worrying about exact span values
pub(crate) fn normalize_spans_in_expr(expr: &mut syntax::Expr) {
  use syntax::Expr::*;
  match expr {
    Variable(ident, span) => {
      normalize_spans_in_identifier(ident);
      *span = syntax::SourceSpan::dummy();
    }
    IntConst(_, span) | UIntConst(_, span) | FloatConst(_, span) | DoubleConst(_, span) | BoolConst(_, span) => {
      *span = syntax::SourceSpan::dummy();
    }
    Binary(_, left, right, span) => {
      normalize_spans_in_expr(left);
      normalize_spans_in_expr(right);
      *span = syntax::SourceSpan::dummy();
    }
    Unary(_, expr, span) => {
      normalize_spans_in_expr(expr);
      *span = syntax::SourceSpan::dummy();
    }
    Assignment(left, _, right, span) => {
      normalize_spans_in_expr(left);
      normalize_spans_in_expr(right);
      *span = syntax::SourceSpan::dummy();
    }
    Bracket(expr, array_spec, span) => {
      normalize_spans_in_expr(expr);
      normalize_spans_in_array_specifier(array_spec);
      *span = syntax::SourceSpan::dummy();
    }
    FunCall(fun_ident, args, span) => {
      normalize_spans_in_fun_identifier(fun_ident);
      for arg in args {
        normalize_spans_in_expr(arg);
      }
      *span = syntax::SourceSpan::dummy();
    }
    Dot(expr, ident, span) => {
      normalize_spans_in_expr(expr);
      normalize_spans_in_identifier(ident);
      *span = syntax::SourceSpan::dummy();
    }
    PostInc(expr, span) | PostDec(expr, span) => {
      normalize_spans_in_expr(expr);
      *span = syntax::SourceSpan::dummy();
    }
    Ternary(cond, true_expr, false_expr, span) => {
      normalize_spans_in_expr(cond);
      normalize_spans_in_expr(true_expr);
      normalize_spans_in_expr(false_expr);
      *span = syntax::SourceSpan::dummy();
    }
    Comma(left, right, span) => {
      normalize_spans_in_expr(left);
      normalize_spans_in_expr(right);
      *span = syntax::SourceSpan::dummy();
    }
  }
}

fn normalize_spans_in_identifier(ident: &mut syntax::Identifier) {
  ident.span = syntax::SourceSpan::dummy();
}

// Normalize spans in FunIdentifier
fn normalize_spans_in_fun_identifier(fun_ident: &mut syntax::FunIdentifier) {
  use syntax::FunIdentifier::*;
  match fun_ident {
    Identifier(ident) => {
      normalize_spans_in_identifier(ident);
    }
    Expr(expr) => {
      normalize_spans_in_expr(expr);
    }
  }
}

// Normalize spans in ArraySpecifierDimension
fn normalize_spans_in_array_specifier_dimension(dim: &mut syntax::ArraySpecifierDimension) {
  use syntax::ArraySpecifierDimension::*;
  match dim {
    Unsized => {}
    ExplicitlySized(expr) => {
      normalize_spans_in_expr(expr);
    }
  }
}

// Normalize spans in ArraySpecifier
fn normalize_spans_in_array_specifier(spec: &mut syntax::ArraySpecifier) {
  for dim in &mut spec.dimensions.0 {
    normalize_spans_in_array_specifier_dimension(dim);
  }
}

// Normalize spans in Initializer
fn normalize_spans_in_initializer(init: &mut syntax::Initializer) {
  use syntax::Initializer::*;
  match init {
    Simple(expr) => {
      normalize_spans_in_expr(expr);
    }
    List(list) => {
      for item in &mut list.0 {
        normalize_spans_in_initializer(item);
      }
    }
  }
}

// Normalize spans in SingleDeclaration
fn normalize_spans_in_single_declaration(decl: &mut syntax::SingleDeclaration) {
  if let Some(ref mut init) = decl.initializer {
    normalize_spans_in_initializer(init);
  }
  if let Some(ref mut spec) = decl.array_specifier {
    normalize_spans_in_array_specifier(spec);
  }
  if let Some(ref mut name) = decl.name {
    normalize_spans_in_identifier(name);
  }
}

// Normalize spans in Statement
fn normalize_spans_in_statement(stmt: &mut syntax::Statement) {
  use syntax::Statement::*;
  match stmt {
    Simple(simple) => {
      normalize_spans_in_simple_statement(simple);
    }
    Compound(compound) => {
      normalize_spans_in_compound_statement(compound);
    }
  }
}

// Normalize spans in SimpleStatement
fn normalize_spans_in_simple_statement(stmt: &mut syntax::SimpleStatement) {
  use syntax::SimpleStatement::*;
  match stmt {
    Declaration(decl) => {
      normalize_spans_in_declaration(decl);
    }
    Expression(Some(ref mut expr)) => {
      normalize_spans_in_expr(expr);
    }
    Expression(None) => {}
    Selection(sel) => {
      normalize_spans_in_selection_statement(sel);
    }
    Switch(sw) => {
      normalize_spans_in_switch_statement(sw);
    }
    Iteration(iter) => {
      normalize_spans_in_iteration_statement(iter);
    }
    Jump(jump) => {
      normalize_spans_in_jump_statement(jump);
    }
    CaseLabel(case) => {
      normalize_spans_in_case_label(case);
    }
  }
}

// Normalize spans in CaseLabel
fn normalize_spans_in_case_label(case: &mut syntax::CaseLabel) {
  use syntax::CaseLabel::*;
  match case {
    Case(expr) => {
      normalize_spans_in_expr(expr);
    }
    Def => {}
  }
}

// Normalize spans in CompoundStatement
fn normalize_spans_in_compound_statement(stmt: &mut syntax::CompoundStatement) {
  for stmt in &mut stmt.statement_list {
    normalize_spans_in_statement(stmt);
  }
}

// Normalize spans in Declaration
fn normalize_spans_in_declaration(decl: &mut syntax::Declaration) {
  use syntax::Declaration::*;
  match decl {
    FunctionPrototype(fp) => {
      normalize_spans_in_function_prototype(fp);
    }
    InitDeclaratorList(list) => {
      normalize_spans_in_single_declaration(&mut list.head);
      for item in &mut list.tail {
        normalize_spans_in_single_declaration_no_type(item);
      }
    }
    Precision(_, _) => {}
    Block(block) => {
      normalize_spans_in_block(block);
    }
    Global(_, idents) => {
      for ident in idents {
        normalize_spans_in_identifier(ident);
      }
    }
  }
}

// Normalize spans in SingleDeclarationNoType
fn normalize_spans_in_single_declaration_no_type(decl: &mut syntax::SingleDeclarationNoType) {
  normalize_spans_in_arrayed_identifier(&mut decl.ident);
  if let Some(ref mut init) = decl.initializer {
    normalize_spans_in_initializer(init);
  }
}

// Normalize spans in StructFieldSpecifier
fn normalize_spans_in_struct_field_specifier(field: &mut syntax::StructFieldSpecifier) {
  for ident in &mut field.identifiers.0 {
    normalize_spans_in_arrayed_identifier(ident);
  }
}

// Normalize spans in Block
fn normalize_spans_in_block(block: &mut syntax::Block) {
  normalize_spans_in_type_qualifier(&mut block.qualifier);
  normalize_spans_in_identifier(&mut block.name);
  for field in &mut block.fields {
    normalize_spans_in_struct_field_specifier(field);
  }
  if let Some(ref mut ident) = block.identifier {
    normalize_spans_in_arrayed_identifier(ident);
  }
}

// Normalize spans in ArrayedIdentifier
fn normalize_spans_in_arrayed_identifier(ident: &mut syntax::ArrayedIdentifier) {
  normalize_spans_in_identifier(&mut ident.ident);
  if let Some(ref mut spec) = ident.array_spec {
    normalize_spans_in_array_specifier(spec);
  }
}

// Normalize spans in SelectionStatement
fn normalize_spans_in_selection_statement(stmt: &mut syntax::SelectionStatement) {
  normalize_spans_in_expr(&mut stmt.cond);
  use syntax::SelectionRestStatement::*;
  match &mut stmt.rest {
    Statement(if_stmt) => {
      normalize_spans_in_statement(if_stmt);
    }
    Else(if_stmt, else_stmt) => {
      normalize_spans_in_statement(if_stmt);
      normalize_spans_in_statement(else_stmt);
    }
  }
}

// Normalize spans in SwitchStatement
fn normalize_spans_in_switch_statement(stmt: &mut syntax::SwitchStatement) {
  normalize_spans_in_expr(&mut stmt.head);
  for stmt in &mut stmt.body {
    normalize_spans_in_statement(stmt);
  }
}

// Normalize spans in IterationStatement
fn normalize_spans_in_iteration_statement(stmt: &mut syntax::IterationStatement) {
  use syntax::IterationStatement::*;
  match stmt {
    While(cond, body) => {
      normalize_spans_in_condition(cond);
      normalize_spans_in_statement(body);
    }
    DoWhile(body, cond) => {
      normalize_spans_in_statement(body);
      normalize_spans_in_expr(cond);
    }
    For(init, rest, body) => {
      normalize_spans_in_for_init_statement(init);
      normalize_spans_in_for_rest_statement(rest);
      normalize_spans_in_statement(body);
    }
  }
}

// Normalize spans in Condition
fn normalize_spans_in_condition(cond: &mut syntax::Condition) {
  use syntax::Condition::*;
  match cond {
    Expr(expr) => {
      normalize_spans_in_expr(expr);
    }
    Assignment(_, ident, ref mut init) => {
      normalize_spans_in_identifier(ident);
      normalize_spans_in_initializer(init);
    }
  }
}

// Normalize spans in ForInitStatement
fn normalize_spans_in_for_init_statement(init: &mut syntax::ForInitStatement) {
  use syntax::ForInitStatement::*;
  match init {
    Declaration(decl) => {
      normalize_spans_in_declaration(decl);
    }
    Expression(Some(ref mut expr)) => {
      normalize_spans_in_expr(expr);
    }
    Expression(None) => {}
  }
}

// Normalize spans in ForRestStatement
fn normalize_spans_in_for_rest_statement(rest: &mut syntax::ForRestStatement) {
  if let Some(ref mut cond) = rest.condition {
    normalize_spans_in_condition(cond);
  }
  if let Some(ref mut expr) = rest.post_expr {
    normalize_spans_in_expr(expr);
  }
}

// Normalize spans in JumpStatement
fn normalize_spans_in_jump_statement(stmt: &mut syntax::JumpStatement) {
  use syntax::JumpStatement::*;
  match stmt {
    Return(Some(expr)) => {
      normalize_spans_in_expr(expr);
    }
    Return(None) | Continue | Break | Discard => {}
  }
}

// Normalize spans in FunctionParameterDeclaration
fn normalize_spans_in_function_parameter_declaration(param: &mut syntax::FunctionParameterDeclaration) {
  use syntax::FunctionParameterDeclaration::*;
  match param {
    Named(_, declarator) => {
      normalize_spans_in_arrayed_identifier(&mut declarator.ident);
    }
    Unnamed(_, _) => {
      // Unnamed parameters don't have identifiers to normalize
    }
  }
}

// Normalize spans in FunctionPrototype
fn normalize_spans_in_function_prototype(fp: &mut syntax::FunctionPrototype) {
  normalize_spans_in_identifier(&mut fp.name);
  for param in &mut fp.parameters {
    normalize_spans_in_function_parameter_declaration(param);
  }
}

// Normalize spans in FunctionDefinition
fn normalize_spans_in_function_definition(fd: &mut syntax::FunctionDefinition) {
  fd.span = syntax::SourceSpan::dummy();
  normalize_spans_in_function_prototype(&mut fd.prototype);
  normalize_spans_in_compound_statement(&mut fd.statement);
}

// Normalize spans in ExternalDeclaration
fn normalize_spans_in_external_declaration(ed: &mut syntax::ExternalDeclaration) {
  use syntax::ExternalDeclaration::*;
  match ed {
    FunctionDefinition(fd) => {
      normalize_spans_in_function_definition(fd);
    }
    Declaration(decl) => {
      normalize_spans_in_declaration(decl);
    }
    Preprocessor(pp) => {
      normalize_spans_in_preprocessor(pp);
    }
  }
}

// Normalize spans in TranslationUnit
fn normalize_spans_in_translation_unit(tu: &mut syntax::TranslationUnit) {
  for ed in &mut tu.0.0 {
    normalize_spans_in_external_declaration(ed);
  }
}

// Normalize spans in LayoutQualifier
fn normalize_spans_in_layout_qualifier(lq: &mut syntax::LayoutQualifier) {
  for spec in &mut lq.ids.0 {
    normalize_spans_in_layout_qualifier_spec(spec);
  }
}

// Normalize spans in LayoutQualifierSpec
fn normalize_spans_in_layout_qualifier_spec(spec: &mut syntax::LayoutQualifierSpec) {
  use syntax::LayoutQualifierSpec::*;
  match spec {
    Identifier(ident, Some(ref mut expr)) => {
      normalize_spans_in_identifier(ident);
      normalize_spans_in_expr(expr);
    }
    Identifier(ident, None) => {
      normalize_spans_in_identifier(ident);
    }
    Shared => {}
  }
}


// Normalize spans in TypeQualifier
fn normalize_spans_in_type_qualifier(tq: &mut syntax::TypeQualifier) {
  for spec in &mut tq.qualifiers.0 {
    normalize_spans_in_type_qualifier_spec(spec);
  }
}

// Normalize spans in TypeQualifierSpec
fn normalize_spans_in_type_qualifier_spec(spec: &mut syntax::TypeQualifierSpec) {
  use syntax::TypeQualifierSpec::*;
  match spec {
    Storage(_) => {}
    Layout(ref mut lq) => {
      normalize_spans_in_layout_qualifier(lq);
    }
    Precision(_) => {}
    Interpolation(_) => {}
    Invariant => {}
    Precise => {}
  }
}

// Normalize spans in StructSpecifier
fn normalize_spans_in_struct_specifier(ss: &mut syntax::StructSpecifier) {
  // TypeName is just a String, no spans to normalize
  for field in &mut ss.fields.0 {
    normalize_spans_in_struct_field_specifier(field);
  }
}

// Normalize spans in Preprocessor
fn normalize_spans_in_preprocessor(pp: &mut syntax::Preprocessor) {
  use syntax::Preprocessor::*;
  match pp {
    Define(ref mut def) => normalize_spans_in_preprocessor_define(def),
    IfDef(ref mut def) => normalize_spans_in_identifier(&mut def.ident),
    IfNDef(ref mut def) => normalize_spans_in_identifier(&mut def.ident),
    Undef(ref mut def) => normalize_spans_in_identifier(&mut def.name),
    _ => {} // Other variants don't have Identifier fields
  }
}

// Normalize spans in PreprocessorDefine
fn normalize_spans_in_preprocessor_define(def: &mut syntax::PreprocessorDefine) {
  use syntax::PreprocessorDefine::*;
  match def {
    ObjectLike { ref mut ident, .. } => {
      normalize_spans_in_identifier(ident);
    }
    FunctionLike { ref mut ident, ref mut args, .. } => {
      normalize_spans_in_identifier(ident);
      for arg in args {
        normalize_spans_in_identifier(arg);
      }
    }
  }
}

// Macro to assert parsed result with span normalization
macro_rules! assert_parsed_eq {
  ($parser:expr, $input:expr, $expected:expr) => {
    {
      let (remaining, mut result) = $parser(span($input)).map(extract_result).unwrap();
      // Try to normalize based on type - this is a bit hacky but works
      if let Ok(_) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(expr) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
          normalize_spans_in_expr(&mut result);
        })) {
          expr
        } else if let Ok(stmt) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
          normalize_spans_in_statement(&mut result);
        })) {
          stmt
        } else if let Ok(decl) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
          normalize_spans_in_declaration(&mut result);
        })) {
          decl
        } else {
          Ok(())
        }
      })) {
        // Normalization succeeded, compare
        assert_eq!((remaining, result), $expected);
      } else {
        // Couldn't normalize, just compare directly
        assert_eq!((remaining, result), $expected);
      }
    }
  };
}

#[test]
fn parse_uniline_comment() {
  assert_eq!(comment(span("// lol")).map(extract_result), Ok(("", " lol")));
  assert_eq!(comment(span("// lol\nfoo")).map(extract_result), Ok(("foo", " lol")));
  assert_eq!(comment(span("// lol\\\nfoo")).map(extract_result), Ok(("", " lol\\\nfoo")));
  assert_eq!(
    comment(span("// lol   \\\n   foo\n")).map(extract_result),
    Ok(("", " lol   \\\n   foo"))
  );
}

#[test]
fn parse_multiline_comment() {
  assert_eq!(comment(span("/* lol\nfoo\n*/bar")).map(extract_result), Ok(("bar", " lol\nfoo\n")));
}

#[test]
fn parse_unsigned_suffix() {
  assert_eq!(unsigned_suffix(span("u")).map(extract_result), Ok(("", 'u')));
  assert_eq!(unsigned_suffix(span("U")).map(extract_result), Ok(("", 'U')));
}

#[test]
fn parse_nonzero_digits() {
  assert_eq!(nonzero_digits(span("3")).map(extract_result), Ok(("", "3")));
  assert_eq!(nonzero_digits(span("12345953")).map(extract_result), Ok(("", "12345953")));
}

#[test]
fn parse_decimal_lit() {
  assert_eq!(decimal_lit(span("3")).map(extract_result), Ok(("", Ok(3))));
  assert_eq!(decimal_lit(span("3")).map(extract_result), Ok(("", Ok(3))));
  assert_eq!(decimal_lit(span("13")).map(extract_result), Ok(("", Ok(13))));
  assert_eq!(decimal_lit(span("42")).map(extract_result), Ok(("", Ok(42))));
  assert_eq!(decimal_lit(span("123456")).map(extract_result), Ok(("", Ok(123456))));
}

#[test]
fn parse_octal_lit() {
  assert_eq!(octal_lit(span("0")).map(extract_result), Ok(("", Ok(0o0))));
  assert_eq!(octal_lit(span("03 ")).map(extract_result), Ok((" ", Ok(0o3))));
  assert_eq!(octal_lit(span("012 ")).map(extract_result), Ok((" ", Ok(0o12))));
  assert_eq!(octal_lit(span("07654321 ")).map(extract_result), Ok((" ", Ok(0o7654321))));
}

#[test]
fn parse_hexadecimal_lit() {
  assert_eq!(hexadecimal_lit(span("0x3 ")).map(extract_result), Ok((" ", Ok(0x3))));
  assert_eq!(hexadecimal_lit(span("0x0123789")).map(extract_result), Ok(("", Ok(0x0123789))));
  assert_eq!(hexadecimal_lit(span("0xABCDEF")).map(extract_result), Ok(("", Ok(0xabcdef))));
  assert_eq!(hexadecimal_lit(span("0xabcdef")).map(extract_result), Ok(("", Ok(0xabcdef))));
}

#[test]
fn parse_integral_lit() {
  assert_eq!(integral_lit(span("0")).map(extract_result), Ok(("", 0)));
  assert_eq!(integral_lit(span("3")).map(extract_result), Ok(("", 3)));
  assert_eq!(integral_lit(span("3 ")).map(extract_result), Ok((" ", 3)));
  assert_eq!(integral_lit(span("03 ")).map(extract_result), Ok((" ", 3)));
  assert_eq!(integral_lit(span("076556 ")).map(extract_result), Ok((" ", 0o76556)));
  assert_eq!(integral_lit(span("012 ")).map(extract_result), Ok((" ", 0o12)));
  assert_eq!(integral_lit(span("0x3 ")).map(extract_result), Ok((" ", 0x3)));
  assert_eq!(integral_lit(span("0x9ABCDEF")).map(extract_result), Ok(("", 0x9ABCDEF)));
  assert_eq!(integral_lit(span("0x9abcdef")).map(extract_result), Ok(("", 0x9abcdef)));
  assert_eq!(integral_lit(span("0xffffffff")).map(extract_result), Ok(("", 0xffffffffu32 as i32)));
}

#[test]
fn parse_integral_neg_lit() {
  assert_eq!(integral_lit(span("-3")).map(extract_result), Ok(("", -3)));
  assert_eq!(integral_lit(span("-3 ")).map(extract_result), Ok((" ", -3)));
  assert_eq!(integral_lit(span("-03 ")).map(extract_result), Ok((" ", -3)));
  assert_eq!(integral_lit(span("-076556 ")).map(extract_result), Ok((" ", -0o76556)));
  assert_eq!(integral_lit(span("-012 ")).map(extract_result), Ok((" ", -0o12)));
  assert_eq!(integral_lit(span("-0x3 ")).map(extract_result), Ok((" ", -0x3)));
  assert_eq!(integral_lit(span("-0x9ABCDEF")).map(extract_result), Ok(("", -0x9ABCDEF)));
  assert_eq!(integral_lit(span("-0x9abcdef")).map(extract_result), Ok(("", -0x9abcdef)));
}

#[test]
fn parse_unsigned_lit() {
  assert_eq!(unsigned_lit(span("0xffffffffU")).map(extract_result), Ok(("", 0xffffffff as u32)));
  assert_eq!(unsigned_lit(span("-1u")).map(extract_result), Ok(("", 0xffffffff as u32)));
  assert!(unsigned_lit(span("0xfffffffffU")).is_err());
}

#[test]
fn parse_float_lit() {
  assert_eq!(float_lit(span("0.;")).map(extract_result), Ok((";", 0.)));
  assert_eq!(float_lit(span(".0;")).map(extract_result), Ok((";", 0.)));
  assert_eq!(float_lit(span(".035 ")).map(extract_result), Ok((" ", 0.035)));
  assert_eq!(float_lit(span("0. ")).map(extract_result), Ok((" ", 0.)));
  assert_eq!(float_lit(span("0.035 ")).map(extract_result), Ok((" ", 0.035)));
  assert_eq!(float_lit(span(".035f")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(float_lit(span("0.f")).map(extract_result), Ok(("", 0.)));
  assert_eq!(float_lit(span("314.f")).map(extract_result), Ok(("", 314.)));
  assert_eq!(float_lit(span("0.035f")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(float_lit(span(".035F")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(float_lit(span("0.F")).map(extract_result), Ok(("", 0.)));
  assert_eq!(float_lit(span("0.035F")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(float_lit(span("1.03e+34 ")).map(extract_result), Ok((" ", 1.03e+34)));
  assert_eq!(float_lit(span("1.03E+34 ")).map(extract_result), Ok((" ", 1.03E+34)));
  assert_eq!(float_lit(span("1.03e-34 ")).map(extract_result), Ok((" ", 1.03e-34)));
  assert_eq!(float_lit(span("1.03E-34 ")).map(extract_result), Ok((" ", 1.03E-34)));
  assert_eq!(float_lit(span("1.03e+34f")).map(extract_result), Ok(("", 1.03e+34)));
  assert_eq!(float_lit(span("1.03E+34f")).map(extract_result), Ok(("", 1.03E+34)));
  assert_eq!(float_lit(span("1.03e-34f")).map(extract_result), Ok(("", 1.03e-34)));
  assert_eq!(float_lit(span("1.03E-34f")).map(extract_result), Ok(("", 1.03E-34)));
  assert_eq!(float_lit(span("1.03e+34F")).map(extract_result), Ok(("", 1.03e+34)));
  assert_eq!(float_lit(span("1.03E+34F")).map(extract_result), Ok(("", 1.03E+34)));
  assert_eq!(float_lit(span("1.03e-34F")).map(extract_result), Ok(("", 1.03e-34)));
  assert_eq!(float_lit(span("1.03E-34F")).map(extract_result), Ok(("", 1.03E-34)));
}

#[test]
fn parse_float_neg_lit() {
  assert_eq!(float_lit(span("-.035 ")).map(extract_result), Ok((" ", -0.035)));
  assert_eq!(float_lit(span("-0. ")).map(extract_result), Ok((" ", -0.)));
  assert_eq!(float_lit(span("-0.035 ")).map(extract_result), Ok((" ", -0.035)));
  assert_eq!(float_lit(span("-.035f")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(float_lit(span("-0.f")).map(extract_result), Ok(("", -0.)));
  assert_eq!(float_lit(span("-0.035f")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(float_lit(span("-.035F")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(float_lit(span("-0.F")).map(extract_result), Ok(("", -0.)));
  assert_eq!(float_lit(span("-0.035F")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(float_lit(span("-1.03e+34 ")).map(extract_result), Ok((" ", -1.03e+34)));
  assert_eq!(float_lit(span("-1.03E+34 ")).map(extract_result), Ok((" ", -1.03E+34)));
  assert_eq!(float_lit(span("-1.03e-34 ")).map(extract_result), Ok((" ", -1.03e-34)));
  assert_eq!(float_lit(span("-1.03E-34 ")).map(extract_result), Ok((" ", -1.03E-34)));
  assert_eq!(float_lit(span("-1.03e+34f")).map(extract_result), Ok(("", -1.03e+34)));
  assert_eq!(float_lit(span("-1.03E+34f")).map(extract_result), Ok(("", -1.03E+34)));
  assert_eq!(float_lit(span("-1.03e-34f")).map(extract_result), Ok(("", -1.03e-34)));
  assert_eq!(float_lit(span("-1.03E-34f")).map(extract_result), Ok(("", -1.03E-34)));
  assert_eq!(float_lit(span("-1.03e+34F")).map(extract_result), Ok(("", -1.03e+34)));
  assert_eq!(float_lit(span("-1.03E+34F")).map(extract_result), Ok(("", -1.03E+34)));
  assert_eq!(float_lit(span("-1.03e-34F")).map(extract_result), Ok(("", -1.03e-34)));
  assert_eq!(float_lit(span("-1.03E-34F")).map(extract_result), Ok(("", -1.03E-34)));
}

#[test]
fn parse_double_lit() {
  assert_eq!(double_lit(span("0.;")).map(extract_result), Ok((";", 0.)));
  assert_eq!(double_lit(span(".0;")).map(extract_result), Ok((";", 0.)));
  assert_eq!(double_lit(span(".035 ")).map(extract_result), Ok((" ", 0.035)));
  assert_eq!(double_lit(span("0. ")).map(extract_result), Ok((" ", 0.)));
  assert_eq!(double_lit(span("0.035 ")).map(extract_result), Ok((" ", 0.035)));
  assert_eq!(double_lit(span("0.lf")).map(extract_result), Ok(("", 0.)));
  assert_eq!(double_lit(span("0.035lf")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(double_lit(span(".035lf")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(double_lit(span(".035LF")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(double_lit(span("0.LF")).map(extract_result), Ok(("", 0.)));
  assert_eq!(double_lit(span("0.035LF")).map(extract_result), Ok(("", 0.035)));
  assert_eq!(double_lit(span("1.03e+34lf")).map(extract_result), Ok(("", 1.03e+34)));
  assert_eq!(double_lit(span("1.03E+34lf")).map(extract_result), Ok(("", 1.03E+34)));
  assert_eq!(double_lit(span("1.03e-34lf")).map(extract_result), Ok(("", 1.03e-34)));
  assert_eq!(double_lit(span("1.03E-34lf")).map(extract_result), Ok(("", 1.03E-34)));
  assert_eq!(double_lit(span("1.03e+34LF")).map(extract_result), Ok(("", 1.03e+34)));
  assert_eq!(double_lit(span("1.03E+34LF")).map(extract_result), Ok(("", 1.03E+34)));
  assert_eq!(double_lit(span("1.03e-34LF")).map(extract_result), Ok(("", 1.03e-34)));
  assert_eq!(double_lit(span("1.03E-34LF")).map(extract_result), Ok(("", 1.03E-34)));
}

#[test]
fn parse_double_neg_lit() {
  assert_eq!(double_lit(span("-0.;")).map(extract_result), Ok((";", -0.)));
  assert_eq!(double_lit(span("-.0;")).map(extract_result), Ok((";", -0.)));
  assert_eq!(double_lit(span("-.035 ")).map(extract_result), Ok((" ", -0.035)));
  assert_eq!(double_lit(span("-0. ")).map(extract_result), Ok((" ", -0.)));
  assert_eq!(double_lit(span("-0.035 ")).map(extract_result), Ok((" ", -0.035)));
  assert_eq!(double_lit(span("-0.lf")).map(extract_result), Ok(("", -0.)));
  assert_eq!(double_lit(span("-0.035lf")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(double_lit(span("-.035lf")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(double_lit(span("-.035LF")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(double_lit(span("-0.LF")).map(extract_result), Ok(("", -0.)));
  assert_eq!(double_lit(span("-0.035LF")).map(extract_result), Ok(("", -0.035)));
  assert_eq!(double_lit(span("-1.03e+34lf")).map(extract_result), Ok(("", -1.03e+34)));
  assert_eq!(double_lit(span("-1.03E+34lf")).map(extract_result), Ok(("", -1.03E+34)));
  assert_eq!(double_lit(span("-1.03e-34lf")).map(extract_result), Ok(("", -1.03e-34)));
  assert_eq!(double_lit(span("-1.03E-34lf")).map(extract_result), Ok(("", -1.03E-34)));
  assert_eq!(double_lit(span("-1.03e+34LF")).map(extract_result), Ok(("", -1.03e+34)));
  assert_eq!(double_lit(span("-1.03E+34LF")).map(extract_result), Ok(("", -1.03E+34)));
  assert_eq!(double_lit(span("-1.03e-34LF")).map(extract_result), Ok(("", -1.03e-34)));
  assert_eq!(double_lit(span("-1.03E-34LF")).map(extract_result), Ok(("", -1.03E-34)));
}

#[test]
fn parse_bool_lit() {
  assert_eq!(bool_lit(span("false")).map(extract_result), Ok(("", false)));
  assert_eq!(bool_lit(span("true")).map(extract_result), Ok(("", true)));
}

#[test]
fn parse_identifier() {
  let mut result_a = identifier(span("a")).map(extract_result).unwrap().1;
  normalize_spans_in_identifier(&mut result_a);
  assert_eq!(result_a, syntax::Identifier { name: "a".into(), span: syntax::SourceSpan::dummy() });
  let mut result_ab = identifier(span("ab_cd")).map(extract_result).unwrap().1;
  normalize_spans_in_identifier(&mut result_ab);
  assert_eq!(result_ab, syntax::Identifier { name: "ab_cd".into(), span: syntax::SourceSpan::dummy() });
  let mut result_Ab = identifier(span("Ab_cd")).map(extract_result).unwrap().1;
  normalize_spans_in_identifier(&mut result_Ab);
  assert_eq!(result_Ab, syntax::Identifier { name: "Ab_cd".into(), span: syntax::SourceSpan::dummy() });
  let mut result_Ab8 = identifier(span("Ab_c8d")).map(extract_result).unwrap().1;
  normalize_spans_in_identifier(&mut result_Ab8);
  assert_eq!(result_Ab8, syntax::Identifier { name: "Ab_c8d".into(), span: syntax::SourceSpan::dummy() });
  let mut result_Ab89 = identifier(span("Ab_c8d9")).map(extract_result).unwrap().1;
  normalize_spans_in_identifier(&mut result_Ab89);
  assert_eq!(result_Ab89, syntax::Identifier { name: "Ab_c8d9".into(), span: syntax::SourceSpan::dummy() });
}

#[test]
fn parse_unary_op_add() {
  assert_eq!(unary_op(span("+ ")).map(extract_result), Ok((" ", syntax::UnaryOp::Add)));
}

#[test]
fn parse_unary_op_minus() {
  assert_eq!(unary_op(span("- ")).map(extract_result), Ok((" ", syntax::UnaryOp::Minus)));
}

#[test]
fn parse_unary_op_not() {
  assert_eq!(unary_op(span("!")).map(extract_result), Ok(("", syntax::UnaryOp::Not)));
}

#[test]
fn parse_unary_op_complement() {
  assert_eq!(unary_op(span("~")).map(extract_result), Ok(("", syntax::UnaryOp::Complement)));
}

#[test]
fn parse_unary_op_inc() {
  assert_eq!(unary_op(span("++")).map(extract_result), Ok(("", syntax::UnaryOp::Inc)));
}

#[test]
fn parse_unary_op_dec() {
  assert_eq!(unary_op(span("--")).map(extract_result), Ok(("", syntax::UnaryOp::Dec)));
}

#[test]
fn parse_array_specifier_dimension_unsized() {
  assert_eq!(
    array_specifier_dimension(span("[]")).map(extract_result),
    Ok(("", syntax::ArraySpecifierDimension::Unsized))
  );
  assert_eq!(
    array_specifier_dimension(span("[ ]")).map(extract_result),
    Ok(("", syntax::ArraySpecifierDimension::Unsized))
  );
  assert_eq!(
    array_specifier_dimension(span("[\n]")).map(extract_result),
    Ok(("", syntax::ArraySpecifierDimension::Unsized))
  );
}

#[test]
fn parse_array_specifier_dimension_sized() {
  let ix = syntax::Expr::IntConst(0, syntax::SourceSpan::dummy());

  let (remaining, mut result) = array_specifier_dimension(span("[0]")).map(extract_result).unwrap();
  normalize_spans_in_array_specifier_dimension(&mut result);
  assert_eq!((remaining, result), ("", syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(ix.clone()))));
  let (remaining, mut result) = array_specifier_dimension(span("[\n0   \t]")).map(extract_result).unwrap();
  normalize_spans_in_array_specifier_dimension(&mut result);
  assert_eq!((remaining, result), ("", syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(ix))));
}

#[test]
fn parse_array_specifier_unsized() {
  assert_eq!(
    array_specifier(span("[]")).map(extract_result),
    Ok((
      "",
      syntax::ArraySpecifier {
        dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::Unsized])
      }
    ))
  )
}

#[test]
fn parse_array_specifier_sized() {
  let ix = syntax::Expr::IntConst(123, syntax::SourceSpan::dummy());

  let (remaining, mut result) = array_specifier(span("[123]")).map(extract_result).unwrap();
  normalize_spans_in_array_specifier(&mut result);
  assert_eq!((remaining, result), ("", syntax::ArraySpecifier {
    dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::ExplicitlySized(
      Box::new(ix)
    )])
  }));
}

#[test]
fn parse_array_specifier_sized_multiple() {
  let a = syntax::Expr::IntConst(2, syntax::SourceSpan::dummy());
  let b = syntax::Expr::IntConst(100, syntax::SourceSpan::dummy());
  let d = syntax::Expr::IntConst(5, syntax::SourceSpan::dummy());

  let (remaining, mut result) = array_specifier(span("[2][100][][5]")).map(extract_result).unwrap();
  normalize_spans_in_array_specifier(&mut result);
  assert_eq!((remaining, result), ("", syntax::ArraySpecifier {
    dimensions: syntax::NonEmpty(vec![
      syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(a)),
      syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(b)),
      syntax::ArraySpecifierDimension::Unsized,
      syntax::ArraySpecifierDimension::ExplicitlySized(Box::new(d)),
    ])
  }));
}

#[test]
fn parse_precise_qualifier() {
  assert_eq!(precise_qualifier(span("precise ")).map(extract_result), Ok((" ", ())));
}

#[test]
fn parse_invariant_qualifier() {
  assert_eq!(invariant_qualifier(span("invariant ")).map(extract_result), Ok((" ", ())));
}

#[test]
fn parse_interpolation_qualifier() {
  assert_eq!(
    interpolation_qualifier(span("smooth ")).map(extract_result),
    Ok((" ", syntax::InterpolationQualifier::Smooth))
  );
  assert_eq!(
    interpolation_qualifier(span("flat ")).map(extract_result),
    Ok((" ", syntax::InterpolationQualifier::Flat))
  );
  assert_eq!(
    interpolation_qualifier(span("noperspective ")).map(extract_result),
    Ok((" ", syntax::InterpolationQualifier::NoPerspective))
  );
}

#[test]
fn parse_precision_qualifier() {
  assert_eq!(
    precision_qualifier(span("highp ")).map(extract_result),
    Ok((" ", syntax::PrecisionQualifier::High))
  );
  assert_eq!(
    precision_qualifier(span("mediump ")).map(extract_result),
    Ok((" ", syntax::PrecisionQualifier::Medium))
  );
  assert_eq!(
    precision_qualifier(span("lowp ")).map(extract_result),
    Ok((" ", syntax::PrecisionQualifier::Low))
  );
}

#[test]
fn parse_storage_qualifier() {
  assert_eq!(
    storage_qualifier(span("const ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Const))
  );
  assert_eq!(
    storage_qualifier(span("inout ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::InOut))
  );
  assert_eq!(
    storage_qualifier(span("in ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::In))
  );
  assert_eq!(
    storage_qualifier(span("out ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Out))
  );
  assert_eq!(
    storage_qualifier(span("centroid ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Centroid))
  );
  assert_eq!(
    storage_qualifier(span("patch ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Patch))
  );
  assert_eq!(
    storage_qualifier(span("sample ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Sample))
  );
  assert_eq!(
    storage_qualifier(span("uniform ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Uniform))
  );
  assert_eq!(
    storage_qualifier(span("attribute ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Attribute))
  );
  assert_eq!(
    storage_qualifier(span("varying ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Varying))
  );
  assert_eq!(
    storage_qualifier(span("buffer ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Buffer))
  );
  assert_eq!(
    storage_qualifier(span("shared ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Shared))
  );
  assert_eq!(
    storage_qualifier(span("coherent ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Coherent))
  );
  assert_eq!(
    storage_qualifier(span("volatile ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Volatile))
  );
  assert_eq!(
    storage_qualifier(span("restrict ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::Restrict))
  );
  assert_eq!(
    storage_qualifier(span("readonly ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::ReadOnly))
  );
  assert_eq!(
    storage_qualifier(span("writeonly ")).map(extract_result),
    Ok((" ", syntax::StorageQualifier::WriteOnly))
  );
  assert_eq!(
    storage_qualifier(span("subroutine a")).map(extract_result),
    Ok((" a", syntax::StorageQualifier::Subroutine(vec![])))
  );

  let a = syntax::TypeName("vec3".to_owned());
  let b = syntax::TypeName("float".to_owned());
  let c = syntax::TypeName("dmat43".to_owned());
  let types = vec![a, b, c];
  assert_eq!(
    storage_qualifier(span("subroutine (  vec3 , float \\\n, dmat43)")).map(extract_result),
    Ok(("", syntax::StorageQualifier::Subroutine(types)))
  );
}

#[test]
fn parse_layout_qualifier_std430() {
  let expected = syntax::LayoutQualifier {
    ids: syntax::NonEmpty(vec![syntax::LayoutQualifierSpec::Identifier(
      "std430".into(),
      None,
    )]),
  };

  let (remaining, mut result) = layout_qualifier(span("layout (std430)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout  (std430   )")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout \n\t (  std430  )")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout(std430)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_layout_qualifier_shared() {
  let expected = syntax::LayoutQualifier {
    ids: syntax::NonEmpty(vec![syntax::LayoutQualifierSpec::Shared]),
  };

  let (remaining, mut result) = layout_qualifier(span("layout (shared)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout ( shared )")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout(shared)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_layout_qualifier_list() {
  let id_0 = syntax::LayoutQualifierSpec::Shared;
  let id_1 = syntax::LayoutQualifierSpec::Identifier("std140".into(), None);
  let id_2 = syntax::LayoutQualifierSpec::Identifier(
    "max_vertices".into(),
    Some(Box::new(syntax::Expr::IntConst(3, syntax::SourceSpan::dummy()))),
  );
  let expected = syntax::LayoutQualifier {
    ids: syntax::NonEmpty(vec![id_0, id_1, id_2]),
  };

  let (remaining, mut result) = layout_qualifier(span("layout (shared, std140, max_vertices = 3)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout(shared,std140,max_vertices=3)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = layout_qualifier(span("layout\n\n\t (    shared , std140, max_vertices= 3)")).map(extract_result).unwrap();
  normalize_spans_in_layout_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
}

#[test]
fn parse_type_qualifier() {
  let storage_qual = syntax::TypeQualifierSpec::Storage(syntax::StorageQualifier::Const);
  let id_0 = syntax::LayoutQualifierSpec::Shared;
  let id_1 = syntax::LayoutQualifierSpec::Identifier("std140".into(), None);
  let id_2 = syntax::LayoutQualifierSpec::Identifier(
    "max_vertices".into(),
    Some(Box::new(syntax::Expr::IntConst(3, syntax::SourceSpan::dummy()))),
  );
  let layout_qual = syntax::TypeQualifierSpec::Layout(syntax::LayoutQualifier {
    ids: syntax::NonEmpty(vec![id_0, id_1, id_2]),
  });
  let expected = syntax::TypeQualifier {
    qualifiers: syntax::NonEmpty(vec![storage_qual, layout_qual]),
  };

  let (remaining, mut result) = type_qualifier(span("const layout (shared, std140, max_vertices = 3)")).map(extract_result).unwrap();
  normalize_spans_in_type_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = type_qualifier(span("const layout(shared,std140,max_vertices=3)")).map(extract_result).unwrap();
  normalize_spans_in_type_qualifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_struct_field_specifier() {
  let expected = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec4,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["foo".into()]),
  };

  let (remaining, mut result) = struct_field_specifier(span("vec4 foo;")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_field_specifier(span("vec4     foo ; ")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), (" ", expected.clone()));
}

#[test]
fn parse_struct_field_specifier_type_name() {
  let expected = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::TypeName("S0238_3".into()),
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["x".into()]),
  };

  let (remaining, mut result) = struct_field_specifier(span("S0238_3 x;")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_field_specifier(span("S0238_3     x ;")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
}

#[test]
fn parse_struct_field_specifier_several() {
  let expected = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec4,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["foo".into(), "bar".into(), "zoo".into()]),
  };

  let (remaining, mut result) = struct_field_specifier(span("vec4 foo, bar, zoo;")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_field_specifier(span("vec4     foo , bar  , zoo ;")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_field_specifier(span("vec4     foo , bar  , zoo ;")).map(extract_result).unwrap();
  normalize_spans_in_struct_field_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_struct_specifier_one_field() {
  let field = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec4,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["foo".into()]),
  };
  let expected = syntax::StructSpecifier {
    name: Some("TestStruct".into()),
    fields: syntax::NonEmpty(vec![field]),
  };

  let (remaining, mut result) = struct_specifier(span("struct TestStruct { vec4 foo; }")).map(extract_result).unwrap();
  normalize_spans_in_struct_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_specifier(span("struct      TestStruct \n \n\n {\n    vec4   foo  ;}")).map(extract_result).unwrap();
  normalize_spans_in_struct_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_struct_specifier_multi_fields() {
  let a = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec4,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["foo".into()]),
  };
  let b = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Float,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["bar".into()]),
  };
  let c = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::UInt,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["zoo".into()]),
  };
  let d = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::BVec3,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["foo_BAR_zoo3497_34".into()]),
  };
  let e = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::TypeName("S0238_3".into()),
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["x".into()]),
  };
  let expected = syntax::StructSpecifier {
    name: Some("_TestStruct_934i".into()),
    fields: syntax::NonEmpty(vec![a, b, c, d, e]),
  };

  let (remaining, mut result) = struct_specifier(span(
    "struct _TestStruct_934i { vec4 foo; float bar; uint zoo; bvec3 foo_BAR_zoo3497_34; S0238_3 x; }"
  )).map(extract_result).unwrap();
  normalize_spans_in_struct_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_specifier(span(
    "struct _TestStruct_934i{vec4 foo;float bar;uint zoo;bvec3 foo_BAR_zoo3497_34;S0238_3 x;}"
  )).map(extract_result).unwrap();
  normalize_spans_in_struct_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = struct_specifier(span("struct _TestStruct_934i\n   {  vec4\nfoo ;   \n\t float\n\t\t  bar  ;   \nuint   zoo;    \n bvec3   foo_BAR_zoo3497_34\n\n\t\n\t\n  ; S0238_3 x;}")).map(extract_result).unwrap();
  normalize_spans_in_struct_specifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_type_specifier_non_array() {
  assert_eq!(
    type_specifier_non_array(span("bool")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Bool))
  );
  assert_eq!(
    type_specifier_non_array(span("int")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Int))
  );
  assert_eq!(
    type_specifier_non_array(span("uint")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UInt))
  );
  assert_eq!(
    type_specifier_non_array(span("float")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Float))
  );
  assert_eq!(
    type_specifier_non_array(span("double")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Double))
  );
  assert_eq!(
    type_specifier_non_array(span("vec2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Vec2))
  );
  assert_eq!(
    type_specifier_non_array(span("vec3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Vec3))
  );
  assert_eq!(
    type_specifier_non_array(span("vec4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Vec4))
  );
  assert_eq!(
    type_specifier_non_array(span("dvec2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DVec2))
  );
  assert_eq!(
    type_specifier_non_array(span("dvec3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DVec3))
  );
  assert_eq!(
    type_specifier_non_array(span("dvec4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DVec4))
  );
  assert_eq!(
    type_specifier_non_array(span("bvec2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::BVec2))
  );
  assert_eq!(
    type_specifier_non_array(span("bvec3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::BVec3))
  );
  assert_eq!(
    type_specifier_non_array(span("bvec4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::BVec4))
  );
  assert_eq!(
    type_specifier_non_array(span("ivec2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IVec2))
  );
  assert_eq!(
    type_specifier_non_array(span("ivec3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IVec3))
  );
  assert_eq!(
    type_specifier_non_array(span("ivec4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IVec4))
  );
  assert_eq!(
    type_specifier_non_array(span("uvec2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UVec2))
  );
  assert_eq!(
    type_specifier_non_array(span("uvec3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UVec3))
  );
  assert_eq!(
    type_specifier_non_array(span("uvec4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UVec4))
  );
  assert_eq!(
    type_specifier_non_array(span("mat2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat2))
  );
  assert_eq!(
    type_specifier_non_array(span("mat3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat3))
  );
  assert_eq!(
    type_specifier_non_array(span("mat4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat4))
  );
  assert_eq!(
    type_specifier_non_array(span("mat2x2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat2))
  );
  assert_eq!(
    type_specifier_non_array(span("mat2x3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat23))
  );
  assert_eq!(
    type_specifier_non_array(span("mat2x4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat24))
  );
  assert_eq!(
    type_specifier_non_array(span("mat3x2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat32))
  );
  assert_eq!(
    type_specifier_non_array(span("mat3x3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat3))
  );
  assert_eq!(
    type_specifier_non_array(span("mat3x4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat34))
  );
  assert_eq!(
    type_specifier_non_array(span("mat4x2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat42))
  );
  assert_eq!(
    type_specifier_non_array(span("mat4x3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat43))
  );
  assert_eq!(
    type_specifier_non_array(span("mat4x4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Mat4))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat2))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat3))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat4))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat2x2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat2))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat2x3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat23))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat2x4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat24))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat3x2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat32))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat3x3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat3))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat3x4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat34))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat4x2")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat42))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat4x3")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat43))
  );
  assert_eq!(
    type_specifier_non_array(span("dmat4x4")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::DMat4))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler1D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler1D))
  );
  assert_eq!(
    type_specifier_non_array(span("image1D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image1D))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2D))
  );
  assert_eq!(
    type_specifier_non_array(span("image2D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image2D))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler3D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler3D))
  );
  assert_eq!(
    type_specifier_non_array(span("image3D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image3D))
  );
  assert_eq!(
    type_specifier_non_array(span("samplerCube")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::SamplerCube))
  );
  assert_eq!(
    type_specifier_non_array(span("imageCube")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ImageCube))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DRect")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DRect))
  );
  assert_eq!(
    type_specifier_non_array(span("image2DRect")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image2DRect))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler1DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler1DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("image1DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image1DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("image2DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image2DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("samplerBuffer")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::SamplerBuffer))
  );
  assert_eq!(
    type_specifier_non_array(span("imageBuffer")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ImageBuffer))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DMS")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DMS))
  );
  assert_eq!(
    type_specifier_non_array(span("image2DMS")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image2DMS))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DMSArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DMSArray))
  );
  assert_eq!(
    type_specifier_non_array(span("image2DMSArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Image2DMSArray))
  );
  assert_eq!(
    type_specifier_non_array(span("samplerCubeArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::SamplerCubeArray))
  );
  assert_eq!(
    type_specifier_non_array(span("imageCubeArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ImageCubeArray))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler1DShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler1DShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DRectShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DRectShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler1DArrayShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler1DArrayShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("sampler2DArrayShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::Sampler2DArrayShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("samplerCubeShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::SamplerCubeShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("samplerCubeArrayShadow")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::SamplerCubeArrayShadow))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler1D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler1D))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage1D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage1D))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler2D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler2D))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage2D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage2D))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler3D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler3D))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage3D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage3D))
  );
  assert_eq!(
    type_specifier_non_array(span("isamplerCube")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISamplerCube))
  );
  assert_eq!(
    type_specifier_non_array(span("iimageCube")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImageCube))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler2DRect")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler2DRect))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage2DRect")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage2DRect))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler1DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler1DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage1DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage1DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler2DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler2DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage2DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage2DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("isamplerBuffer")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISamplerBuffer))
  );
  assert_eq!(
    type_specifier_non_array(span("iimageBuffer")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImageBuffer))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler2DMS")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler2DMS))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage2DMS")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage2DMS))
  );
  assert_eq!(
    type_specifier_non_array(span("isampler2DMSArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISampler2DMSArray))
  );
  assert_eq!(
    type_specifier_non_array(span("iimage2DMSArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImage2DMSArray))
  );
  assert_eq!(
    type_specifier_non_array(span("isamplerCubeArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::ISamplerCubeArray))
  );
  assert_eq!(
    type_specifier_non_array(span("iimageCubeArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::IImageCubeArray))
  );
  assert_eq!(
    type_specifier_non_array(span("atomic_uint")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::AtomicUInt))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler1D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler1D))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage1D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage1D))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler2D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler2D))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage2D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage2D))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler3D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler3D))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage3D")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage3D))
  );
  assert_eq!(
    type_specifier_non_array(span("usamplerCube")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USamplerCube))
  );
  assert_eq!(
    type_specifier_non_array(span("uimageCube")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImageCube))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler2DRect")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler2DRect))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage2DRect")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage2DRect))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler1DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler1DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage1DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage1DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler2DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler2DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage2DArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage2DArray))
  );
  assert_eq!(
    type_specifier_non_array(span("usamplerBuffer")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USamplerBuffer))
  );
  assert_eq!(
    type_specifier_non_array(span("uimageBuffer")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImageBuffer))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler2DMS")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler2DMS))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage2DMS")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage2DMS))
  );
  assert_eq!(
    type_specifier_non_array(span("usampler2DMSArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USampler2DMSArray))
  );
  assert_eq!(
    type_specifier_non_array(span("uimage2DMSArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImage2DMSArray))
  );
  assert_eq!(
    type_specifier_non_array(span("usamplerCubeArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::USamplerCubeArray))
  );
  assert_eq!(
    type_specifier_non_array(span("uimageCubeArray")).map(extract_result),
    Ok(("", syntax::TypeSpecifierNonArray::UImageCubeArray))
  );
  assert_eq!(
    type_specifier_non_array(span("ReturnType")).map(extract_result),
    Ok((
      "",
      syntax::TypeSpecifierNonArray::TypeName(syntax::TypeName::new("ReturnType").unwrap())
    ))
  );
}

#[test]
fn parse_type_specifier() {
  // TypeSpecifier doesn't have spans, but ArraySpecifier does
  let (remaining, mut result) = type_specifier(span("uint;")).map(extract_result).unwrap();
  if let Some(ref mut spec) = result.array_specifier {
    normalize_spans_in_array_specifier(spec);
  }
  assert_eq!((remaining, result), (";", syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::UInt,
    array_specifier: None
  }));
  let (remaining, mut result) = type_specifier(span("iimage2DMSArray[35];")).map(extract_result).unwrap();
  if let Some(ref mut spec) = result.array_specifier {
    normalize_spans_in_array_specifier(spec);
  }
  assert_eq!((remaining, result), (";", syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::IImage2DMSArray,
    array_specifier: Some(syntax::ArraySpecifier {
      dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::ExplicitlySized(
        Box::new(syntax::Expr::IntConst(35, syntax::SourceSpan::dummy()))
      )])
    })
  }));
}

#[test]
fn parse_fully_specified_type() {
  let ty = syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::IImage2DMSArray,
    array_specifier: None,
  };
  let expected = syntax::FullySpecifiedType {
    qualifier: None,
    ty,
  };

  assert_eq!(
    fully_specified_type(span("iimage2DMSArray;")).map(extract_result),
    Ok((";", expected.clone()))
  );
}

#[test]
fn parse_fully_specified_type_with_qualifier() {
  let qual_spec = syntax::TypeQualifierSpec::Storage(syntax::StorageQualifier::Subroutine(vec![
    "vec2".into(),
    "S032_29k".into(),
  ]));
  let qual = syntax::TypeQualifier {
    qualifiers: syntax::NonEmpty(vec![qual_spec]),
  };
  let ty = syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::IImage2DMSArray,
    array_specifier: None,
  };
  let expected = syntax::FullySpecifiedType {
    qualifier: Some(qual),
    ty,
  };

  assert_eq!(
    fully_specified_type(span("subroutine (vec2, S032_29k) iimage2DMSArray;")).map(extract_result),
    Ok((";", expected.clone()))
  );
  assert_eq!(
    fully_specified_type(span("subroutine (  vec2\t\n \t , \n S032_29k   )\n iimage2DMSArray ;")).map(extract_result),
    Ok((" ;", expected.clone()))
  );
  assert_eq!(
    fully_specified_type(span("subroutine(vec2,S032_29k)iimage2DMSArray;")).map(extract_result),
    Ok((";", expected))
  );
}

#[test]
fn parse_primary_expr_intconst() {
  let (remaining, mut expr) = primary_expr(span("0 ")).map(extract_result).unwrap();
  assert_eq!(remaining, " ");
  normalize_spans_in_expr(&mut expr);
  assert_eq!(expr, syntax::Expr::IntConst(0, syntax::SourceSpan::dummy()));
  let (remaining, mut expr) = primary_expr(span("1 ")).map(extract_result).unwrap();
  assert_eq!(remaining, " ");
  normalize_spans_in_expr(&mut expr);
  assert_eq!(expr, syntax::Expr::IntConst(1, syntax::SourceSpan::dummy()));
}

#[test]
fn parse_primary_expr_uintconst() {
  let (remaining, mut expr) = primary_expr(span("0u ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::UIntConst(0, syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("1u ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::UIntConst(1, syntax::SourceSpan::dummy())));
}

#[test]
fn parse_primary_expr_floatconst() {
  let (remaining, mut expr) = primary_expr(span("0.f ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("1.f ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::FloatConst(1., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("0.F ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("1.F ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::FloatConst(1., syntax::SourceSpan::dummy())));
}

#[test]
fn parse_primary_expr_doubleconst() {
  let (remaining, mut expr) = primary_expr(span("0. ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("1. ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::FloatConst(1., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("0.lf ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::DoubleConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("1.lf ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::DoubleConst(1., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("0.LF ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::DoubleConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("1.LF ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::DoubleConst(1., syntax::SourceSpan::dummy())));
}

#[test]
fn parse_primary_expr_boolconst() {
  let (remaining, mut expr) = primary_expr(span("false")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), ("", syntax::Expr::BoolConst(false, syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("true")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), ("", syntax::Expr::BoolConst(true, syntax::SourceSpan::dummy())));
}

#[test]
fn parse_primary_expr_parens() {
  let (remaining, mut expr) = primary_expr(span("(0)")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), ("", syntax::Expr::IntConst(0, syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("(  0 )")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), ("", syntax::Expr::IntConst(0, syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("(  .0 )")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), ("", syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("(  (.0) )")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), ("", syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy())));
  let (remaining, mut expr) = primary_expr(span("(true) ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ", syntax::Expr::BoolConst(true, syntax::SourceSpan::dummy())));
}

#[test]
fn parse_postfix_function_call_no_args() {
  let fun = syntax::FunIdentifier::Identifier("vec3".into());
  let args = Vec::new();
  let expected = syntax::Expr::FunCall(fun, args, syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("vec3();")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("vec3   (  ) ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ;", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("vec3   (\nvoid\n) ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ;", expected));
}

#[test]
fn parse_postfix_function_call_one_arg() {
  let fun = syntax::FunIdentifier::Identifier("foo".into());
  let args = vec![syntax::Expr::IntConst(0, syntax::SourceSpan::dummy())];
  let expected = syntax::Expr::FunCall(fun, args, syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo(0);")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("foo   ( 0 ) ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ;", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("foo   (\n0\t\n) ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ;", expected));
}

#[test]
fn parse_postfix_function_call_multi_arg() {
  let fun = syntax::FunIdentifier::Identifier("foo".into());
  let args = vec![
    syntax::Expr::IntConst(0, syntax::SourceSpan::dummy()),
    syntax::Expr::BoolConst(false, syntax::SourceSpan::dummy()),
    syntax::Expr::Variable("bar".into(), syntax::SourceSpan::dummy()),
  ];
  let expected = syntax::Expr::FunCall(fun, args, syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo(0, false, bar);")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("foo   ( 0\t, false    ,\t\tbar) ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (" ;", expected));
}

#[test]
fn parse_postfix_expr_bracket() {
  let id = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let array_spec = syntax::ArraySpecifier {
    dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::ExplicitlySized(
      Box::new(syntax::Expr::IntConst(7354, syntax::SourceSpan::dummy())),
    )]),
  };
  let expected = syntax::Expr::Bracket(Box::new(id), array_spec, syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo[7354];")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("foo[\n  7354    ] ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected));
}

#[test]
fn parse_postfix_expr_dot() {
  let foo = Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy()));
  let expected = syntax::Expr::Dot(foo, "bar".into(), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo.bar;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("(foo).bar;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected));
}

#[test]
fn parse_postfix_expr_dot_several() {
  let foo = Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy()));
  let expected = syntax::Expr::Dot(Box::new(syntax::Expr::Dot(foo, "bar".into(), syntax::SourceSpan::dummy())), "zoo".into(), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo.bar.zoo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("(foo).bar.zoo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
  let (remaining, mut expr) = postfix_expr(span("(foo.bar).zoo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected));
}

#[test]
fn parse_postfix_postinc() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::PostInc(Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo++;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_postfix_postdec() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::PostDec(Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = postfix_expr(span("foo--;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_unary_add() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::Unary(syntax::UnaryOp::Add, Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = unary_expr(span("+foo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_unary_minus() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::Unary(syntax::UnaryOp::Minus, Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = unary_expr(span("-foo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_unary_not() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::Unary(syntax::UnaryOp::Not, Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = unary_expr(span("!foo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected));
}

#[test]
fn parse_unary_complement() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::Unary(syntax::UnaryOp::Complement, Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = unary_expr(span("~foo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_unary_inc() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::Unary(syntax::UnaryOp::Inc, Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = unary_expr(span("++foo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_unary_dec() {
  let foo = syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy());
  let expected = syntax::Expr::Unary(syntax::UnaryOp::Dec, Box::new(foo), syntax::SourceSpan::dummy());

  let (remaining, mut expr) = unary_expr(span("--foo;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut expr);
  assert_eq!((remaining, expr), (";", expected.clone()));
}

#[test]
fn parse_expr_float() {
  let (remaining, mut result) = expr(span("314.;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", syntax::Expr::FloatConst(314., syntax::SourceSpan::dummy())));
  let (remaining, mut result) = expr(span("314.f;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", syntax::Expr::FloatConst(314., syntax::SourceSpan::dummy())));
  let (remaining, mut result) = expr(span("314.LF;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", syntax::Expr::DoubleConst(314., syntax::SourceSpan::dummy())));
}

#[test]
fn parse_expr_add_2() {
  let one = Box::new(syntax::Expr::IntConst(1, syntax::SourceSpan::dummy()));
  let expected = syntax::Expr::Binary(syntax::BinaryOp::Add, one.clone(), one, syntax::SourceSpan::dummy());

  let (remaining, mut result) = expr(span("1 + 1;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected.clone()));
  let (remaining, mut result) = expr(span("1+1;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected.clone()));
  let (remaining, mut result) = expr(span("(1 + 1);")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected));
}

#[test]
fn parse_expr_add_3() {
  let one = Box::new(syntax::Expr::UIntConst(1, syntax::SourceSpan::dummy()));
  let two = Box::new(syntax::Expr::UIntConst(2, syntax::SourceSpan::dummy()));
  let three = Box::new(syntax::Expr::UIntConst(3, syntax::SourceSpan::dummy()));
  let expected = syntax::Expr::Binary(
    syntax::BinaryOp::Add,
    Box::new(syntax::Expr::Binary(syntax::BinaryOp::Add, one, two, syntax::SourceSpan::dummy())),
    three,
    syntax::SourceSpan::dummy(),
  );

  let (remaining, mut result) = expr(span("1u + 2u + 3u")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = expr(span("1u + 2u + 3u   ")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), ("   ", expected.clone()));
  let (remaining, mut result) = expr(span("1u+2u+3u")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = expr(span("((1u + 2u) + 3u)")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_expr_add_mult_3() {
  let one = Box::new(syntax::Expr::UIntConst(1, syntax::SourceSpan::dummy()));
  let two = Box::new(syntax::Expr::UIntConst(2, syntax::SourceSpan::dummy()));
  let three = Box::new(syntax::Expr::UIntConst(3, syntax::SourceSpan::dummy()));
  let expected = syntax::Expr::Binary(
    syntax::BinaryOp::Add,
    Box::new(syntax::Expr::Binary(syntax::BinaryOp::Mult, one, two, syntax::SourceSpan::dummy())),
    three,
    syntax::SourceSpan::dummy(),
  );

  let (remaining, mut result) = expr(span("1u * 2u + 3u ;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (" ;", expected.clone()));
  let (remaining, mut result) = expr(span("1u*2u+3u;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected.clone()));
  let (remaining, mut result) = expr(span("(1u * 2u) + 3u;")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected));
}

#[test]
fn parse_expr_add_sub_mult_div() {
  let one = Box::new(syntax::Expr::IntConst(1, syntax::SourceSpan::dummy()));
  let two = Box::new(syntax::Expr::IntConst(2, syntax::SourceSpan::dummy()));
  let three = Box::new(syntax::Expr::IntConst(3, syntax::SourceSpan::dummy()));
  let four = Box::new(syntax::Expr::IntConst(4, syntax::SourceSpan::dummy()));
  let five = Box::new(syntax::Expr::IntConst(5, syntax::SourceSpan::dummy()));
  let six = Box::new(syntax::Expr::IntConst(6, syntax::SourceSpan::dummy()));
  let expected = syntax::Expr::Binary(
    syntax::BinaryOp::Add,
    Box::new(syntax::Expr::Binary(
      syntax::BinaryOp::Mult,
      one,
      Box::new(syntax::Expr::Binary(syntax::BinaryOp::Add, two, three, syntax::SourceSpan::dummy())),
      syntax::SourceSpan::dummy(),
    )),
    Box::new(syntax::Expr::Binary(
      syntax::BinaryOp::Div,
      four,
      Box::new(syntax::Expr::Binary(syntax::BinaryOp::Add, five, six, syntax::SourceSpan::dummy())),
      syntax::SourceSpan::dummy(),
    )),
    syntax::SourceSpan::dummy(),
  );

  let (remaining, mut result) = expr(span("1 * (2 + 3) + 4 / (5 + 6);")).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected.clone()));
}

#[test]
fn parse_complex_expr() {
  let input = "normalize((inverse(view) * vec4(ray.dir, 0.)).xyz);";
  let zero = syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy());
  let ray = syntax::Expr::Variable("ray".into(), syntax::SourceSpan::dummy());
  let raydir = syntax::Expr::Dot(Box::new(ray), "dir".into(), syntax::SourceSpan::dummy());
  let vec4 = syntax::Expr::FunCall(
    syntax::FunIdentifier::Identifier("vec4".into()),
    vec![raydir, zero],
    syntax::SourceSpan::dummy(),
  );
  let view = syntax::Expr::Variable("view".into(), syntax::SourceSpan::dummy());
  let iview = syntax::Expr::FunCall(
    syntax::FunIdentifier::Identifier("inverse".into()),
    vec![view],
    syntax::SourceSpan::dummy(),
  );
  let mul = syntax::Expr::Binary(syntax::BinaryOp::Mult, Box::new(iview), Box::new(vec4), syntax::SourceSpan::dummy());
  let xyz = syntax::Expr::Dot(Box::new(mul), "xyz".into(), syntax::SourceSpan::dummy());
  let normalize = syntax::Expr::FunCall(
    syntax::FunIdentifier::Identifier("normalize".into()),
    vec![xyz],
    syntax::SourceSpan::dummy(),
  );
  let expected = normalize;

  let (remaining, mut result) = expr(span(&input[..])).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected));
}

#[test]
fn parse_function_identifier_typename() {
  let expected = syntax::FunIdentifier::Identifier("foo".into());
  let (remaining, mut result) = function_identifier(span("foo(")).map(extract_result).unwrap();
  normalize_spans_in_fun_identifier(&mut result);
  assert_eq!((remaining, result), ("(", expected.clone()));
  let (remaining, mut result) = function_identifier(span("foo\n\t(")).map(extract_result).unwrap();
  normalize_spans_in_fun_identifier(&mut result);
  assert_eq!((remaining, result), ("(", expected.clone()));
  let (remaining, mut result) = function_identifier(span("foo\n (")).map(extract_result).unwrap();
  normalize_spans_in_fun_identifier(&mut result);
  assert_eq!((remaining, result), ("(", expected));
}

#[test]
fn parse_function_identifier_cast() {
  let expected = syntax::FunIdentifier::Identifier("vec3".into());
  let (remaining, mut result) = function_identifier(span("vec3(")).map(extract_result).unwrap();
  normalize_spans_in_fun_identifier(&mut result);
  assert_eq!((remaining, result), ("(", expected.clone()));
  let (remaining, mut result) = function_identifier(span("vec3 (")).map(extract_result).unwrap();
  normalize_spans_in_fun_identifier(&mut result);
  assert_eq!((remaining, result), ("(", expected.clone()));
  let (remaining, mut result) = function_identifier(span("vec3\t\n\n \t (")).map(extract_result).unwrap();
  normalize_spans_in_fun_identifier(&mut result);
  assert_eq!((remaining, result), ("(", expected));
}

#[test]
fn parse_function_identifier_cast_array_unsized() {
    let expected = syntax::FunIdentifier::Expr(Box::new(syntax::Expr::Bracket(
      Box::new(syntax::Expr::Variable("vec3".into(), syntax::SourceSpan::dummy())),
    syntax::ArraySpecifier {
      dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::Unsized]),
    },
    syntax::SourceSpan::dummy(),
  )));

  let (remaining, mut result) = function_identifier(span("vec3[](")).map(extract_result).unwrap();
  if let syntax::FunIdentifier::Expr(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("(", expected.clone()));
  let (remaining, mut result) = function_identifier(span("vec3  [\t\n](")).map(extract_result).unwrap();
  if let syntax::FunIdentifier::Expr(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("(", expected));
}

#[test]
fn parse_function_identifier_cast_array_sized() {
    let expected = syntax::FunIdentifier::Expr(Box::new(syntax::Expr::Bracket(
      Box::new(syntax::Expr::Variable("vec3".into(), syntax::SourceSpan::dummy())),
    syntax::ArraySpecifier {
      dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::ExplicitlySized(
        Box::new(syntax::Expr::IntConst(12, syntax::SourceSpan::dummy())),
      )]),
    },
    syntax::SourceSpan::dummy(),
  )));

  let (remaining, mut result) = function_identifier(span("vec3[12](")).map(extract_result).unwrap();
  if let syntax::FunIdentifier::Expr(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("(", expected.clone()));
  let (remaining, mut result) = function_identifier(span("vec3  [\t 12\n](")).map(extract_result).unwrap();
  if let syntax::FunIdentifier::Expr(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("(", expected));
}

#[test]
fn parse_void() {
  assert_eq!(void(span("void ")).map(extract_result), Ok((" ", ())));
}

#[test]
fn parse_assignment_op_equal() {
  assert_eq!(assignment_op(span("= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Equal)));
}

#[test]
fn parse_assignment_op_mult() {
  assert_eq!(assignment_op(span("*= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Mult)));
}

#[test]
fn parse_assignment_op_div() {
  assert_eq!(assignment_op(span("/= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Div)));
}

#[test]
fn parse_assignment_op_mod() {
  assert_eq!(assignment_op(span("%= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Mod)));
}

#[test]
fn parse_assignment_op_add() {
  assert_eq!(assignment_op(span("+= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Add)));
}

#[test]
fn parse_assignment_op_sub() {
  assert_eq!(assignment_op(span("-= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Sub)));
}

#[test]
fn parse_assignment_op_lshift() {
  assert_eq!(
    assignment_op(span("<<= ")).map(extract_result),
    Ok((" ", syntax::AssignmentOp::LShift))
  );
}

#[test]
fn parse_assignment_op_rshift() {
  assert_eq!(
    assignment_op(span(">>= ")).map(extract_result),
    Ok((" ", syntax::AssignmentOp::RShift))
  );
}

#[test]
fn parse_assignment_op_and() {
  assert_eq!(assignment_op(span("&= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::And)));
}

#[test]
fn parse_assignment_op_xor() {
  assert_eq!(assignment_op(span("^= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Xor)));
}

#[test]
fn parse_assignment_op_or() {
  assert_eq!(assignment_op(span("|= ")).map(extract_result), Ok((" ", syntax::AssignmentOp::Or)));
}

#[test]
fn parse_expr_statement() {
  let expected = Some(syntax::Expr::Assignment(
    Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy())),
    syntax::AssignmentOp::Equal,
    Box::new(syntax::Expr::FloatConst(314., syntax::SourceSpan::dummy())),
    syntax::SourceSpan::dummy(),
  ));

  let (remaining, mut result) = expr_statement(span("foo = 314.f;")).map(extract_result).unwrap();
  if let Some(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = expr_statement(span("foo=314.f;")).map(extract_result).unwrap();
  if let Some(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = expr_statement(span("foo\n\t=  \n314.f;")).map(extract_result).unwrap();
  if let Some(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_function_prototype() {
  let rt = syntax::FullySpecifiedType {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec3,
      array_specifier: None,
    },
  };
  let arg0_ty = syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::Vec2,
    array_specifier: None,
  };
  let arg0 = syntax::FunctionParameterDeclaration::Unnamed(None, arg0_ty);
  let qual_spec = syntax::TypeQualifierSpec::Storage(syntax::StorageQualifier::Out);
  let qual = syntax::TypeQualifier {
    qualifiers: syntax::NonEmpty(vec![qual_spec]),
  };
  let arg1 = syntax::FunctionParameterDeclaration::Named(
    Some(qual),
    syntax::FunctionParameterDeclarator {
      ty: syntax::TypeSpecifier {
        ty: syntax::TypeSpecifierNonArray::Float,
        array_specifier: None,
      },
      ident: "the_arg".into(),
    },
  );
  let fp = syntax::FunctionPrototype {
    ty: rt,
    name: "foo".into(),
    parameters: vec![arg0, arg1],
  };
  let expected = syntax::Declaration::FunctionPrototype(fp);

  let (remaining, mut result) = declaration(span("vec3 foo(vec2, out float the_arg);")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("vec3 \nfoo ( vec2\n, out float \n\tthe_arg )\n;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("vec3 foo(vec2,out float the_arg);")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_init_declarator_list_single() {
  let ty = syntax::FullySpecifiedType {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Int,
      array_specifier: None,
    },
  };
  let sd = syntax::SingleDeclaration {
    ty,
    name: Some("foo".into()),
    array_specifier: None,
    initializer: Some(syntax::Initializer::Simple(Box::new(
      syntax::Expr::IntConst(34, syntax::SourceSpan::dummy()),
    ))),
  };
  let idl = syntax::InitDeclaratorList {
    head: sd,
    tail: Vec::new(),
  };
  let expected = syntax::Declaration::InitDeclaratorList(idl);

  let (remaining, mut result) = declaration(span("int foo = 34;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("int foo=34;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("int    \t  \nfoo =\t34  ;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_init_declarator_list_complex() {
  let ty = syntax::FullySpecifiedType {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Int,
      array_specifier: None,
    },
  };
  let sd = syntax::SingleDeclaration {
    ty,
    name: Some("foo".into()),
    array_specifier: None,
    initializer: Some(syntax::Initializer::Simple(Box::new(
      syntax::Expr::IntConst(34, syntax::SourceSpan::dummy()),
    ))),
  };
  let sdnt = syntax::SingleDeclarationNoType {
    ident: "bar".into(),
    initializer: Some(syntax::Initializer::Simple(Box::new(
      syntax::Expr::IntConst(12, syntax::SourceSpan::dummy()),
    ))),
  };
  let expected = syntax::Declaration::InitDeclaratorList(syntax::InitDeclaratorList {
    head: sd,
    tail: vec![sdnt],
  });

  let (remaining, mut result) = declaration(span("int foo = 34, bar = 12;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("int foo=34,bar=12;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("int    \t  \nfoo =\t34 \n,\tbar=      12\n ;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_precision_low() {
  let qual = syntax::PrecisionQualifier::Low;
  let ty = syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::Float,
    array_specifier: None,
  };
  let expected = syntax::Declaration::Precision(qual, ty);

  let (remaining, mut result) = declaration(span("precision lowp float;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_precision_medium() {
  let qual = syntax::PrecisionQualifier::Medium;
  let ty = syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::Float,
    array_specifier: None,
  };
  let expected = syntax::Declaration::Precision(qual, ty);

  let (remaining, mut result) = declaration(span("precision mediump float;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_precision_high() {
  let qual = syntax::PrecisionQualifier::High;
  let ty = syntax::TypeSpecifier {
    ty: syntax::TypeSpecifierNonArray::Float,
    array_specifier: None,
  };
  let expected = syntax::Declaration::Precision(qual, ty);

  let (remaining, mut result) = declaration(span("precision highp float;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_uniform_block() {
  let qual_spec = syntax::TypeQualifierSpec::Storage(syntax::StorageQualifier::Uniform);
  let qual = syntax::TypeQualifier {
    qualifiers: syntax::NonEmpty(vec![qual_spec]),
  };
  let f0 = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Float,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["a".into()]),
  };
  let f1 = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec3,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["b".into()]),
  };
  let f2 = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::TypeName("foo".into()),
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["c".into(), "d".into()]),
  };
  let expected = syntax::Declaration::Block(syntax::Block {
    qualifier: qual,
    name: "UniformBlockTest".into(),
    fields: vec![f0, f1, f2],
    identifier: None,
  });

  let (remaining, mut result) = declaration(span("uniform UniformBlockTest { float a; vec3 b; foo c, d; };")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("uniform   \nUniformBlockTest\n {\n \t float   a  \n; \nvec3 b\n; foo \nc\n, \nd\n;\n }\n\t\n\t\t \t;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_declaration_buffer_block() {
  let qual_spec = syntax::TypeQualifierSpec::Storage(syntax::StorageQualifier::Buffer);
  let qual = syntax::TypeQualifier {
    qualifiers: syntax::NonEmpty(vec![qual_spec]),
  };
  let f0 = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Float,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["a".into()]),
  };
  let f1 = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::Vec3,
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec![syntax::ArrayedIdentifier::new(
      "b",
      Some(syntax::ArraySpecifier {
        dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::Unsized]),
      }),
    )]),
  };
  let f2 = syntax::StructFieldSpecifier {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::TypeName("foo".into()),
      array_specifier: None,
    },
    identifiers: syntax::NonEmpty(vec!["c".into(), "d".into()]),
  };
  let expected = syntax::Declaration::Block(syntax::Block {
    qualifier: qual,
    name: "UniformBlockTest".into(),
    fields: vec![f0, f1, f2],
    identifier: None,
  });

  let (remaining, mut result) = declaration(span("buffer UniformBlockTest { float a; vec3 b[]; foo c, d; };")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = declaration(span("buffer   \nUniformBlockTest\n {\n \t float   a  \n; \nvec3 b   [   ]\n; foo \nc\n, \nd\n;\n }\n\t\n\t\t \t;")).map(extract_result).unwrap();
  normalize_spans_in_declaration(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_selection_statement_if() {
  let cond = syntax::Expr::Binary(
    syntax::BinaryOp::LT,
    Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy())),
    Box::new(syntax::Expr::IntConst(10, syntax::SourceSpan::dummy())),
    syntax::SourceSpan::dummy(),
  );
  let ret = Box::new(syntax::Expr::BoolConst(false, syntax::SourceSpan::dummy()));
  let st = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Jump(
    syntax::JumpStatement::Return(Some(ret)),
  )));
  let body = syntax::Statement::Compound(Box::new(syntax::CompoundStatement {
    statement_list: vec![st],
  }));
  let rest = syntax::SelectionRestStatement::Statement(Box::new(body));
  let expected = syntax::SelectionStatement {
    cond: Box::new(cond),
    rest,
  };

  let (remaining, mut result) = selection_statement(span("if (foo < 10) { return false; }K")).map(extract_result).unwrap();
  normalize_spans_in_selection_statement(&mut result);
  assert_eq!((remaining, result), ("K", expected.clone()));
  let (remaining, mut result) = selection_statement(span("if \n(foo<10\n) \t{return false;}K")).map(extract_result).unwrap();
  normalize_spans_in_selection_statement(&mut result);
  assert_eq!((remaining, result), ("K", expected));
}

#[test]
fn parse_selection_statement_if_else() {
  let cond = syntax::Expr::Binary(
    syntax::BinaryOp::LT,
    Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy())),
    Box::new(syntax::Expr::IntConst(10, syntax::SourceSpan::dummy())),
    syntax::SourceSpan::dummy(),
  );
  let if_ret = Box::new(syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy()));
  let if_st = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Jump(
    syntax::JumpStatement::Return(Some(if_ret)),
  )));
  let if_body = syntax::Statement::Compound(Box::new(syntax::CompoundStatement {
    statement_list: vec![if_st],
  }));
  let else_ret = Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy()));
  let else_st = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Jump(
    syntax::JumpStatement::Return(Some(else_ret)),
  )));
  let else_body = syntax::Statement::Compound(Box::new(syntax::CompoundStatement {
    statement_list: vec![else_st],
  }));
  let rest = syntax::SelectionRestStatement::Else(Box::new(if_body), Box::new(else_body));
  let expected = syntax::SelectionStatement {
    cond: Box::new(cond),
    rest,
  };

  let (remaining, mut result) = selection_statement(span("if (foo < 10) { return 0.f; } else { return foo; }")).map(extract_result).unwrap();
  normalize_spans_in_selection_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = selection_statement(span("if \n(foo<10\n) \t{return 0.f\t;\n\n}\n else{\n\t return foo   ;}")).map(extract_result).unwrap();
  normalize_spans_in_selection_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_switch_statement_empty() {
  let head = Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy()));
  let expected = syntax::SwitchStatement {
    head,
    body: Vec::new(),
  };

  let (remaining, mut result) = switch_statement(span("switch (foo) {}")).map(extract_result).unwrap();
  normalize_spans_in_switch_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = switch_statement(span("switch(foo){}")).map(extract_result).unwrap();
  normalize_spans_in_switch_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = switch_statement(span("switch\n\n (  foo  \t   \n) { \n\n   }")).map(extract_result).unwrap();
  normalize_spans_in_switch_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_switch_statement_cases() {
  let head = Box::new(syntax::Expr::Variable("foo".into(), syntax::SourceSpan::dummy()));
  let case0 = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::CaseLabel(
    syntax::CaseLabel::Case(Box::new(syntax::Expr::IntConst(0, syntax::SourceSpan::dummy()))),
  )));
  let case1 = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::CaseLabel(
    syntax::CaseLabel::Case(Box::new(syntax::Expr::IntConst(1, syntax::SourceSpan::dummy()))),
  )));
  let ret = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Jump(
    syntax::JumpStatement::Return(Some(Box::new(syntax::Expr::UIntConst(12, syntax::SourceSpan::dummy())))),
  )));
  let expected = syntax::SwitchStatement {
    head,
    body: vec![case0, case1, ret],
  };

  let (remaining, mut result) = switch_statement(span("switch (foo) { case 0: case 1: return 12u; }")).map(extract_result).unwrap();
  normalize_spans_in_switch_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
}

#[test]
fn parse_case_label_def() {
  // CaseLabel::Def has no spans to normalize
  assert_eq!(case_label(span("default:")).map(extract_result), Ok(("", syntax::CaseLabel::Def)));
  assert_eq!(case_label(span("default   :")).map(extract_result), Ok(("", syntax::CaseLabel::Def)));
}

#[test]
fn parse_case_label() {
  let (remaining, mut result) = case_label(span("case 3:")).map(extract_result).unwrap();
  if let syntax::CaseLabel::Case(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  let expected = syntax::CaseLabel::Case(Box::new(syntax::Expr::IntConst(3, syntax::SourceSpan::dummy())));
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = case_label(span("case\n\t 3   :")).map(extract_result).unwrap();
  if let syntax::CaseLabel::Case(ref mut expr) = result {
    normalize_spans_in_expr(expr);
  }
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_iteration_statement_while_empty() {
  let cond = syntax::Condition::Expr(Box::new(syntax::Expr::Binary(
    syntax::BinaryOp::GTE,
    Box::new(syntax::Expr::Variable("a".into(), syntax::SourceSpan::dummy())),
    Box::new(syntax::Expr::Variable("b".into(), syntax::SourceSpan::dummy())),
    syntax::SourceSpan::dummy(),
  )));
  let st = syntax::Statement::Compound(Box::new(syntax::CompoundStatement {
    statement_list: Vec::new(),
  }));
  let expected = syntax::IterationStatement::While(cond, Box::new(st));

  let (remaining, mut result) = iteration_statement(span("while (a >= b) {}")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = iteration_statement(span("while(a>=b){}")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = iteration_statement(span("while (  a >=\n\tb  )\t  {   \n}")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_iteration_statement_do_while_empty() {
  let st = syntax::Statement::Compound(Box::new(syntax::CompoundStatement {
    statement_list: Vec::new(),
  }));
  let cond = Box::new(syntax::Expr::Binary(
    syntax::BinaryOp::GTE,
    Box::new(syntax::Expr::Variable("a".into(), syntax::SourceSpan::dummy())),
    Box::new(syntax::Expr::Variable("b".into(), syntax::SourceSpan::dummy())),
    syntax::SourceSpan::dummy(),
  ));
  let expected = syntax::IterationStatement::DoWhile(Box::new(st), cond);

  let (remaining, mut result) = iteration_statement(span("do {} while (a >= b);")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = iteration_statement(span("do{}while(a>=b);")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = iteration_statement(span("do \n {\n} while (  a >=\n\tb  )\t  \n;")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_iteration_statement_for_empty() {
  let init = syntax::ForInitStatement::Declaration(Box::new(
    syntax::Declaration::InitDeclaratorList(syntax::InitDeclaratorList {
      head: syntax::SingleDeclaration {
        ty: syntax::FullySpecifiedType {
          qualifier: None,
          ty: syntax::TypeSpecifier {
            ty: syntax::TypeSpecifierNonArray::Float,
            array_specifier: None,
          },
        },
        name: Some("i".into()),
        array_specifier: None,
        initializer: Some(syntax::Initializer::Simple(Box::new(
          syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy()),
        ))),
      },
      tail: Vec::new(),
    }),
  ));
  let rest = syntax::ForRestStatement {
    condition: Some(syntax::Condition::Expr(Box::new(syntax::Expr::Binary(
      syntax::BinaryOp::LTE,
      Box::new(syntax::Expr::Variable("i".into(), syntax::SourceSpan::dummy())),
      Box::new(syntax::Expr::FloatConst(10., syntax::SourceSpan::dummy())),
      syntax::SourceSpan::dummy(),
    )))),
    post_expr: Some(Box::new(syntax::Expr::Unary(
      syntax::UnaryOp::Inc,
      Box::new(syntax::Expr::Variable("i".into(), syntax::SourceSpan::dummy())),
      syntax::SourceSpan::dummy(),
    ))),
  };
  let st = syntax::Statement::Compound(Box::new(syntax::CompoundStatement {
    statement_list: Vec::new(),
  }));
  let expected = syntax::IterationStatement::For(init, rest, Box::new(st));

  let (remaining, mut result) = iteration_statement(span("for (float i = 0.f; i <= 10.f; ++i) {}")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = iteration_statement(span("for(float i=0.f;i<=10.f;++i){}")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = iteration_statement(span("for\n\t (  \t\n\nfloat \ni \t=\n0.f\n;\ni\t<=  10.f; \n++i\n)\n{\n}")).map(extract_result).unwrap();
  normalize_spans_in_iteration_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_jump_continue() {
  assert_eq!(
    jump_statement(span("continue;")).map(extract_result),
    Ok(("", syntax::JumpStatement::Continue))
  );
}

#[test]
fn parse_jump_break() {
  assert_eq!(
    jump_statement(span("break;")).map(extract_result),
    Ok(("", syntax::JumpStatement::Break))
  );
}

#[test]
fn parse_jump_return() {
  let (remaining, mut result) = jump_statement(span("return 3;")).map(extract_result).unwrap();
  normalize_spans_in_jump_statement(&mut result);
  let expected = syntax::JumpStatement::Return(Some(Box::new(syntax::Expr::IntConst(3, syntax::SourceSpan::dummy()))));
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_jump_empty_return() {
  let expected = syntax::SimpleStatement::Jump(syntax::JumpStatement::Return(None));
  let (remaining, mut result) = simple_statement(span("return;")).map(extract_result).unwrap();
  normalize_spans_in_simple_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_jump_discard() {
  assert_eq!(
    jump_statement(span("discard;")).map(extract_result),
    Ok(("", syntax::JumpStatement::Discard))
  );
}

#[test]
fn parse_simple_statement_return() {
  let e = syntax::Expr::BoolConst(false, syntax::SourceSpan::dummy());
  let expected = syntax::SimpleStatement::Jump(syntax::JumpStatement::Return(Some(Box::new(e))));

  let (remaining, mut result) = simple_statement(span("return false;")).map(extract_result).unwrap();
  normalize_spans_in_simple_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_compound_statement_empty() {
  let expected = syntax::CompoundStatement {
    statement_list: Vec::new(),
  };

  let (remaining, mut result) = compound_statement(span("{}")).map(extract_result).unwrap();
  normalize_spans_in_compound_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_compound_statement() {
  let st0 = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Selection(
    syntax::SelectionStatement {
      cond: Box::new(syntax::Expr::BoolConst(true, syntax::SourceSpan::dummy())),
      rest: syntax::SelectionRestStatement::Statement(Box::new(syntax::Statement::Compound(
        Box::new(syntax::CompoundStatement {
          statement_list: Vec::new(),
        }),
      ))),
    },
  )));
  let st1 = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Declaration(
    syntax::Declaration::InitDeclaratorList(syntax::InitDeclaratorList {
      head: syntax::SingleDeclaration {
        ty: syntax::FullySpecifiedType {
          qualifier: None,
          ty: syntax::TypeSpecifier {
            ty: syntax::TypeSpecifierNonArray::ISampler3D,
            array_specifier: None,
          },
        },
        name: Some("x".into()),
        array_specifier: None,
        initializer: None,
      },
      tail: Vec::new(),
    }),
  )));
  let st2 = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Jump(
    syntax::JumpStatement::Return(Some(Box::new(syntax::Expr::IntConst(42, syntax::SourceSpan::dummy())))),
  )));
  let expected = syntax::CompoundStatement {
    statement_list: vec![st0, st1, st2],
  };

  let (remaining, mut result) = compound_statement(span("{ if (true) {} isampler3D x; return 42 ; }")).map(extract_result).unwrap();
  normalize_spans_in_compound_statement(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = compound_statement(span("{if(true){}isampler3D x;return 42;}")).map(extract_result).unwrap();
  normalize_spans_in_compound_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_function_definition() {
  let rt = syntax::FullySpecifiedType {
    qualifier: None,
    ty: syntax::TypeSpecifier {
      ty: syntax::TypeSpecifierNonArray::IImage2DArray,
      array_specifier: None,
    },
  };
  let fp = syntax::FunctionPrototype {
    ty: rt,
    name: "foo".into(),
    parameters: Vec::new(),
  };
  let st0 = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Jump(
    syntax::JumpStatement::Return(Some(Box::new(syntax::Expr::Variable("bar".into(), syntax::SourceSpan::dummy())))),
  )));
  let expected = syntax::FunctionDefinition {
    prototype: fp,
    statement: syntax::CompoundStatement {
      statement_list: vec![st0],
    },
    span: syntax::SourceSpan::dummy(),
  };

  let (remaining, mut result) = function_definition(span("iimage2DArray foo() { return bar; }")).map(extract_result).unwrap();
  normalize_spans_in_function_definition(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = function_definition(span("iimage2DArray \tfoo\n()\n \n{\n return \nbar\n;}")).map(extract_result).unwrap();
  normalize_spans_in_function_definition(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = function_definition(span("iimage2DArray foo(){return bar;}")).map(extract_result).unwrap();
  normalize_spans_in_function_definition(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_buffer_block_0() {
  let src = include_str!("../data/tests/buffer_block_0.glsl");
  let main_fn = syntax::ExternalDeclaration::FunctionDefinition(syntax::FunctionDefinition {
    prototype: syntax::FunctionPrototype {
      ty: syntax::FullySpecifiedType {
        qualifier: None,
        ty: syntax::TypeSpecifier {
          ty: syntax::TypeSpecifierNonArray::Void,
          array_specifier: None,
        },
      },
      name: "main".into(),
      parameters: Vec::new(),
    },
    statement: syntax::CompoundStatement {
      statement_list: Vec::new(),
    },
    span: syntax::SourceSpan::dummy(),
  });
  let buffer_block =
    syntax::ExternalDeclaration::Declaration(syntax::Declaration::Block(syntax::Block {
      qualifier: syntax::TypeQualifier {
        qualifiers: syntax::NonEmpty(vec![syntax::TypeQualifierSpec::Storage(
          syntax::StorageQualifier::Buffer,
        )]),
      },
      name: "Foo".into(),
      fields: vec![syntax::StructFieldSpecifier {
        qualifier: None,
        ty: syntax::TypeSpecifier {
          ty: syntax::TypeSpecifierNonArray::TypeName("char".into()),
          array_specifier: None,
        },
        identifiers: syntax::NonEmpty(vec![syntax::ArrayedIdentifier::new(
          "tiles",
          Some(syntax::ArraySpecifier {
            dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::Unsized]),
          }),
        )]),
      }],
      identifier: Some("main_tiles".into()),
    }));
  let expected = syntax::TranslationUnit(syntax::NonEmpty(vec![buffer_block, main_fn]));

  let (remaining, mut result) = translation_unit(span(src)).map(extract_result).unwrap();
  normalize_spans_in_translation_unit(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_layout_buffer_block_0() {
  let src = include_str!("../data/tests/layout_buffer_block_0.glsl");
  let layout = syntax::LayoutQualifier {
    ids: syntax::NonEmpty(vec![
      syntax::LayoutQualifierSpec::Identifier(
        "set".into(),
        Some(Box::new(syntax::Expr::IntConst(0, syntax::SourceSpan::dummy()))),
      ),
      syntax::LayoutQualifierSpec::Identifier(
        "binding".into(),
        Some(Box::new(syntax::Expr::IntConst(0, syntax::SourceSpan::dummy()))),
      ),
    ]),
  };
  let type_qual = syntax::TypeQualifier {
    qualifiers: syntax::NonEmpty(vec![
      syntax::TypeQualifierSpec::Layout(layout),
      syntax::TypeQualifierSpec::Storage(syntax::StorageQualifier::Buffer),
    ]),
  };
  let block = syntax::ExternalDeclaration::Declaration(syntax::Declaration::Block(syntax::Block {
    qualifier: type_qual,
    name: "Foo".into(),
    fields: vec![syntax::StructFieldSpecifier {
      qualifier: None,
      ty: syntax::TypeSpecifier {
        ty: syntax::TypeSpecifierNonArray::TypeName("char".into()),
        array_specifier: None,
      },
      identifiers: syntax::NonEmpty(vec!["a".into()]),
    }],
    identifier: Some("foo".into()),
  }));

  let expected = syntax::TranslationUnit(syntax::NonEmpty(vec![block]));

  let (remaining, mut result) = translation_unit(span(src)).map(extract_result).unwrap();
  normalize_spans_in_translation_unit(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_pp_space0() {
  assert_eq!(pp_space0(span("   \\\n  ")).map(extract_result), Ok(("", "   \\\n  ")));
  assert_eq!(pp_space0(span("")).map(extract_result), Ok(("", "")));
}

#[test]
fn parse_pp_version_number() {
  assert_eq!(pp_version_number(span("450")).map(extract_result), Ok(("", 450)));
}

#[test]
fn parse_pp_version_profile() {
  assert_eq!(
    pp_version_profile(span("core")).map(extract_result),
    Ok(("", syntax::PreprocessorVersionProfile::Core))
  );
  assert_eq!(
    pp_version_profile(span("compatibility")).map(extract_result),
    Ok(("", syntax::PreprocessorVersionProfile::Compatibility))
  );
  assert_eq!(
    pp_version_profile(span("es")).map(extract_result),
    Ok(("", syntax::PreprocessorVersionProfile::ES))
  );
}

#[test]
fn parse_pp_version() {
  assert_eq!(
    preprocessor(span("#version 450\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Version(syntax::PreprocessorVersion {
        version: 450,
        profile: None,
      })
    ))
  );

  assert_eq!(
    preprocessor(span("#version 450 core\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Version(syntax::PreprocessorVersion {
        version: 450,
        profile: Some(syntax::PreprocessorVersionProfile::Core)
      })
    ))
  );
}

#[test]
fn parse_pp_version_newline() {
  assert_eq!(
    preprocessor(span("#version 450\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Version(syntax::PreprocessorVersion {
        version: 450,
        profile: None,
      })
    ))
  );

  assert_eq!(
    preprocessor(span("#version 450 core\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Version(syntax::PreprocessorVersion {
        version: 450,
        profile: Some(syntax::PreprocessorVersionProfile::Core)
      })
    ))
  );
}

#[test]
fn parse_pp_define() {
  let expected = |v: &str| {
    (
      "",
      syntax::Preprocessor::Define(syntax::PreprocessorDefine::ObjectLike {
        ident: "test".into(),
        value: v.to_owned(),
      }),
    )
  };

  let (remaining, mut result) = preprocessor(span("#define test 1.0")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), expected("1.0"));
  let (remaining, mut result) = preprocessor(span("#define test \\\n   1.0")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), expected("1.0"));
  let (remaining, mut result) = preprocessor(span("#define test 1.0\n")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), expected("1.0"));

  let expected_test123 = syntax::Preprocessor::Define(syntax::PreprocessorDefine::ObjectLike {
    ident: "test123".into(),
    value: ".0f".to_owned()
  });
  let (remaining, mut result) = preprocessor(span("#define test123 .0f\n")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected_test123));

  let expected_test1 = syntax::Preprocessor::Define(syntax::PreprocessorDefine::ObjectLike {
    ident: "test".into(),
    value: "1".to_owned()
  });
  let (remaining, mut result) = preprocessor(span("#define test 1\n")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected_test1));
}

#[test]
fn parse_pp_define_with_args() {
  let expected = syntax::Preprocessor::Define(syntax::PreprocessorDefine::FunctionLike {
    ident: "add".into(),
    args: vec![
      syntax::Identifier::new("x").unwrap(),
      syntax::Identifier::new("y").unwrap(),
    ],
    value: "(x + y)".to_owned(),
  });

  let (remaining, mut result) = preprocessor(span("#define \\\n add(x, y) \\\n (x + y)")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));

  let (remaining, mut result) = preprocessor(span("#define \\\n add(  x, y  ) \\\n (x + y)")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_pp_define_multiline() {
  let expected = syntax::Preprocessor::Define(syntax::PreprocessorDefine::ObjectLike {
    ident: "foo".into(),
    value: "32".to_owned(),
  });

  let (remaining, mut result) = preprocessor(span(
    r#"#define foo \
       32"#
  )).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_pp_else() {
  assert_eq!(
    preprocessor(span("#    else\n")).map(extract_result),
    Ok(("", syntax::Preprocessor::Else))
  );
}

#[test]
fn parse_pp_elif() {
  assert_eq!(
    preprocessor(span("#   elif \\\n42\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::ElIf(syntax::PreprocessorElIf {
        condition: "42".to_owned()
      })
    ))
  );
}

#[test]
fn parse_pp_endif() {
  assert_eq!(
    preprocessor(span("#\\\nendif")).map(extract_result),
    Ok(("", syntax::Preprocessor::EndIf))
  );
}

#[test]
fn parse_pp_error() {
  assert_eq!(
    preprocessor(span("#error \\\n     some message")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Error(syntax::PreprocessorError {
        message: "some message".to_owned()
      })
    ))
  );
}

#[test]
fn parse_pp_if() {
  assert_eq!(
    preprocessor(span("# \\\nif 42")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::If(syntax::PreprocessorIf {
        condition: "42".to_owned()
      })
    ))
  );
}

#[test]
fn parse_pp_ifdef() {
  let (remaining, mut result) = preprocessor(span("#ifdef       FOO\n")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", syntax::Preprocessor::IfDef(syntax::PreprocessorIfDef {
    ident: syntax::Identifier {
      name: "FOO".to_owned(),
      span: syntax::SourceSpan::dummy(),
    }
  })));
}

#[test]
fn parse_pp_ifndef() {
  let expected = syntax::Preprocessor::IfNDef(syntax::PreprocessorIfNDef {
    ident: syntax::Identifier {
      name: "FOO".to_owned(),
      span: syntax::SourceSpan::dummy(),
    }
  });

  let (remaining, mut result) = preprocessor(span("#\\\nifndef \\\n   FOO\n")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_pp_include() {
  assert_eq!(
    preprocessor(span("#include <filename>\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Include(syntax::PreprocessorInclude {
        path: syntax::Path::Absolute("filename".to_owned())
      })
    ))
  );

  assert_eq!(
      preprocessor(span("#include \\\n\"filename\"\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Include(syntax::PreprocessorInclude {
        path: syntax::Path::Relative("filename".to_owned())
      })
    ))
  );
}

#[test]
fn parse_pp_line() {
  assert_eq!(
    preprocessor(span("#   line \\\n2\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Line(syntax::PreprocessorLine {
        line: 2,
        source_string_number: None,
      })
    ))
  );

  assert_eq!(
    preprocessor(span("#line 2 \\\n 4\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Line(syntax::PreprocessorLine {
        line: 2,
        source_string_number: Some(4),
      })
    ))
  );
}

#[test]
fn parse_pp_pragma() {
  assert_eq!(
    preprocessor(span("#\\\npragma  some   flag")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Pragma(syntax::PreprocessorPragma {
        command: "some   flag".to_owned()
      })
    ))
  );
}

#[test]
fn parse_pp_undef() {
  let expected = syntax::Preprocessor::Undef(syntax::PreprocessorUndef {
    name: syntax::Identifier {
      name: "FOO".to_owned(),
      span: syntax::SourceSpan::dummy(),
    }
  });

  let (remaining, mut result) = preprocessor(span("# undef \\\n FOO")).map(extract_result).unwrap();
  normalize_spans_in_preprocessor(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_pp_extension_name() {
  assert_eq!(
    pp_extension_name(span("all")).map(extract_result),
    Ok(("", syntax::PreprocessorExtensionName::All))
  );
  assert_eq!(
    pp_extension_name(span("GL_foobar_extension ")).map(extract_result),
    Ok((
      " ",
      syntax::PreprocessorExtensionName::Specific("GL_foobar_extension".to_owned())
    ))
  );
}

#[test]
fn parse_pp_extension_behavior() {
  assert_eq!(
    pp_extension_behavior(span("require")).map(extract_result),
    Ok(("", syntax::PreprocessorExtensionBehavior::Require))
  );
  assert_eq!(
    pp_extension_behavior(span("enable")).map(extract_result),
    Ok(("", syntax::PreprocessorExtensionBehavior::Enable))
  );
  assert_eq!(
    pp_extension_behavior(span("warn")).map(extract_result),
    Ok(("", syntax::PreprocessorExtensionBehavior::Warn))
  );
  assert_eq!(
    pp_extension_behavior(span("disable")).map(extract_result),
    Ok(("", syntax::PreprocessorExtensionBehavior::Disable))
  );
}

#[test]
fn parse_pp_extension() {
  assert_eq!(
    preprocessor(span("#extension all: require\n")).map(extract_result),
    Ok((
      "",
      syntax::Preprocessor::Extension(syntax::PreprocessorExtension {
        name: syntax::PreprocessorExtensionName::All,
        behavior: Some(syntax::PreprocessorExtensionBehavior::Require)
      })
    ))
  );
}

#[test]
fn parse_dot_field_expr_array() {
  let src = "a[0].xyz;";
  let expected = syntax::Expr::Dot(
    Box::new(syntax::Expr::Bracket(
      Box::new(syntax::Expr::Variable("a".into(), syntax::SourceSpan::dummy())),
      syntax::ArraySpecifier {
        dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::ExplicitlySized(
          Box::new(syntax::Expr::IntConst(0, syntax::SourceSpan::dummy())),
        )]),
      },
      syntax::SourceSpan::dummy(),
    )),
    "xyz".into(),
    syntax::SourceSpan::dummy(),
  );

  let (remaining, mut result) = expr(span(src)).map(extract_result).unwrap();
  normalize_spans_in_expr(&mut result);
  assert_eq!((remaining, result), (";", expected));
}

#[test]
fn parse_dot_field_expr_statement() {
  let src = "vec3 v = smoothstep(vec3(border_width), vec3(0.0), v_barycenter).zyx;";
  let fun = syntax::FunIdentifier::Identifier("smoothstep".into());
  let args = vec![
    syntax::Expr::FunCall(
      syntax::FunIdentifier::Identifier("vec3".into()),
      vec![syntax::Expr::Variable("border_width".into(), syntax::SourceSpan::dummy())],
      syntax::SourceSpan::dummy(),
    ),
    syntax::Expr::FunCall(
      syntax::FunIdentifier::Identifier("vec3".into()),
      vec![syntax::Expr::FloatConst(0., syntax::SourceSpan::dummy())],
      syntax::SourceSpan::dummy(),
    ),
    syntax::Expr::Variable("v_barycenter".into(), syntax::SourceSpan::dummy()),
  ];
  let ini = syntax::Initializer::Simple(Box::new(syntax::Expr::Dot(
    Box::new(syntax::Expr::FunCall(fun, args, syntax::SourceSpan::dummy())),
    "zyx".into(),
    syntax::SourceSpan::dummy(),
  )));
  let sd = syntax::SingleDeclaration {
    ty: syntax::FullySpecifiedType {
      qualifier: None,
      ty: syntax::TypeSpecifier {
        ty: syntax::TypeSpecifierNonArray::Vec3,
        array_specifier: None,
      },
    },
    name: Some("v".into()),
    array_specifier: None,
    initializer: Some(ini),
  };
  let expected = syntax::Statement::Simple(Box::new(syntax::SimpleStatement::Declaration(
    syntax::Declaration::InitDeclaratorList(syntax::InitDeclaratorList {
      head: sd,
      tail: Vec::new(),
    }),
  )));

  let (remaining, mut result) = statement(span(src)).map(extract_result).unwrap();
  normalize_spans_in_statement(&mut result);
  assert_eq!((remaining, result), ("", expected));
}

#[test]
fn parse_arrayed_identifier() {
  let expected = syntax::ArrayedIdentifier::new(
    "foo",
    syntax::ArraySpecifier {
      dimensions: syntax::NonEmpty(vec![syntax::ArraySpecifierDimension::Unsized]),
    },
  );

  let (remaining, mut result) = arrayed_identifier(span("foo[]")).map(extract_result).unwrap();
  normalize_spans_in_arrayed_identifier(&mut result);
  assert_eq!((remaining, result), ("", expected.clone()));
  let (remaining, mut result) = arrayed_identifier(span("foo \t\n  [\n\t ]")).map(extract_result).unwrap();
  normalize_spans_in_arrayed_identifier(&mut result);
  assert_eq!((remaining, result), ("", expected));
}
