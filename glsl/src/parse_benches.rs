use std::hint::black_box;

use crate::parsers::*;

use test::bench::Bencher;

#[bench]
fn bench_parse_uniline_comment(b: &mut Bencher) {
  b.iter(|| comment(black_box("// lol\\\nfoo")));
}

#[bench]
fn bench_parse_multiline_comment(b: &mut Bencher) {
  b.iter(|| comment(black_box("/* lol\nfoo\n*/bar")));
}

#[bench]
fn bench_parse_unsigned_suffix(b: &mut Bencher) {
  b.iter(|| black_box(unsigned_suffix("u")));
}

#[bench]
fn bench_parse_nonzero_digits(b: &mut Bencher) {
  b.iter(|| black_box(nonzero_digits("12345953")));
}

#[bench]
fn bench_parse_decimal_lit(b: &mut Bencher) {
  b.iter(|| black_box(decimal_lit("123456")));
}

#[bench]
fn bench_parse_octal_lit(b: &mut Bencher) {
  b.iter(|| black_box(octal_lit("07654321 ")));
}

#[bench]
fn bench_parse_hexadecimal_lit(b: &mut Bencher) {
  b.iter(|| black_box(hexadecimal_lit("0xabcdef")));
}

#[bench]
fn bench_parse_integral_lit(b: &mut Bencher) {
  b.iter(|| black_box(integral_lit("0x9abcdef")));
}

#[bench]
fn bench_parse_integral_neg_lit(b: &mut Bencher) {
  b.iter(|| black_box(integral_lit("-0x9abcdef")));
}

#[bench]
fn bench_parse_unsigned_lit(b: &mut Bencher) {
  b.iter(|| black_box(unsigned_lit("0xffffffffU")));
}

#[bench]
fn bench_parse_float_lit(b: &mut Bencher) {
  b.iter(|| black_box(float_lit("1.03e+34f")));
}

#[bench]
fn bench_parse_float_neg_lit(b: &mut Bencher) {
  b.iter(|| black_box(float_lit("-1.03e+34f")));
}

#[bench]
fn bench_parse_double_lit(b: &mut Bencher) {
  b.iter(|| black_box(double_lit("1.03e+34lf")));
}

#[bench]
fn bench_parse_double_neg_lit(b: &mut Bencher) {
  b.iter(|| black_box(double_lit("-1.03e+34lf")));
}

#[bench]
fn bench_parse_bool_lit(b: &mut Bencher) {
  b.iter(|| black_box(bool_lit("false")));
}

#[bench]
fn bench_parse_identifier(b: &mut Bencher) {
  b.iter(|| black_box(identifier("Ab_c8d9")));
}

#[bench]
fn bench_parse_unary_op_add(b: &mut Bencher) {
  b.iter(|| black_box(unary_op("+ ")));
}

#[bench]
fn bench_parse_unary_op_minus(b: &mut Bencher) {
  b.iter(|| black_box(unary_op("- ")));
}

#[bench]
fn bench_parse_unary_op_not(b: &mut Bencher) {
  b.iter(|| black_box(unary_op("!")));
}

#[bench]
fn bench_parse_unary_op_complement(b: &mut Bencher) {
  b.iter(|| black_box(unary_op("~")));
}

#[bench]
fn bench_parse_unary_op_inc(b: &mut Bencher) {
  b.iter(|| black_box(unary_op("++")));
}

#[bench]
fn bench_parse_unary_op_dec(b: &mut Bencher) {
  b.iter(|| black_box(unary_op("--")));
}

#[bench]
fn bench_parse_array_specifier_dimension_unsized(b: &mut Bencher) {
  b.iter(|| black_box(array_specifier_dimension("[\n]")));
}

#[bench]
fn bench_parse_array_specifier_dimension_sized(b: &mut Bencher) {
  b.iter(|| black_box(array_specifier_dimension("[\n0   \t]")));
}

#[bench]
fn bench_parse_array_specifier_unsized(b: &mut Bencher) {
  b.iter(|| black_box(array_specifier("[]")));
}

#[bench]
fn bench_parse_array_specifier_sized(b: &mut Bencher) {
  b.iter(|| black_box(array_specifier("[123]")));
}

#[bench]
fn bench_parse_array_specifier_sized_multiple(b: &mut Bencher) {
  b.iter(|| black_box(array_specifier("[2][100][][5]")));
}

#[bench]
fn bench_parse_precise_qualifier(b: &mut Bencher) {
  b.iter(|| black_box(precise_qualifier("precise ")));
}

#[bench]
fn bench_parse_invariant_qualifier(b: &mut Bencher) {
  b.iter(|| black_box(invariant_qualifier("invariant ")));
}

#[bench]
fn bench_parse_interpolation_qualifier(b: &mut Bencher) {
  b.iter(|| black_box(interpolation_qualifier("noperspective ")));
}

#[bench]
fn bench_parse_precision_qualifier(b: &mut Bencher) {
  b.iter(|| black_box(precision_qualifier("mediump ")));
}

#[bench]
fn bench_parse_storage_qualifier(b: &mut Bencher) {
  b.iter(|| {
    black_box(storage_qualifier(
      "subroutine (  vec3 , float \\\n, dmat43)",
    ))
  });
}

#[bench]
fn bench_parse_layout_qualifier_std430(b: &mut Bencher) {
  b.iter(|| black_box(layout_qualifier("layout(std430)")));
}

#[bench]
fn bench_parse_layout_qualifier_shared(b: &mut Bencher) {
  b.iter(|| black_box(layout_qualifier("layout(shared)")));
}

#[bench]
fn bench_parse_layout_qualifier_list(b: &mut Bencher) {
  b.iter(|| {
    black_box(layout_qualifier(
      "layout\n\n\t (    shared , std140, max_vertices= 3)",
    ))
  });
}

#[bench]
fn bench_parse_type_qualifier(b: &mut Bencher) {
  b.iter(|| {
    black_box(type_qualifier(
      "const layout (shared, std140, max_vertices = 3)",
    ))
  });
}

#[bench]
fn bench_parse_struct_field_specifier(b: &mut Bencher) {
  b.iter(|| black_box(struct_field_specifier("vec4     foo ; ")));
}

#[bench]
fn bench_parse_struct_field_specifier_type_name(b: &mut Bencher) {
  b.iter(|| black_box(struct_field_specifier("S0238_3     x ;")));
}

#[bench]
fn bench_parse_struct_field_specifier_several(b: &mut Bencher) {
  b.iter(|| black_box(struct_field_specifier("vec4     foo , bar  , zoo ;")));
}

#[bench]
fn bench_parse_struct_specifier_one_field(b: &mut Bencher) {
  b.iter(|| {
    black_box(struct_specifier(
      "struct      TestStruct \n \n\n {\n    vec4   foo  ;}",
    ))
  });
}

#[bench]
fn bench_parse_struct_specifier_multi_fields(b: &mut Bencher) {
  b.iter(|| black_box(struct_specifier("struct _TestStruct_934i\n   {  vec4\nfoo ;   \n\t float\n\t\t  bar  ;   \nuint   zoo;    \n bvec3   foo_BAR_zoo3497_34\n\n\t\n\t\n  ; S0238_3 x;}")));
}

#[bench]
fn bench_parse_type_specifier_non_array(b: &mut Bencher) {
  b.iter(|| black_box(type_specifier_non_array("samplerCubeArrayShadow")));
}

#[bench]
fn bench_parse_type_specifier(b: &mut Bencher) {
  b.iter(|| black_box(type_specifier("iimage2DMSArray[35];")));
}

#[bench]
fn bench_parse_fully_specified_type(b: &mut Bencher) {
  b.iter(|| black_box(fully_specified_type("iimage2DMSArray;")));
}

#[bench]
fn bench_parse_fully_specified_type_with_qualifier(b: &mut Bencher) {
  b.iter(|| {
    black_box(fully_specified_type(
      "subroutine(vec2,S032_29k)iimage2DMSArray;",
    ))
  });
}

#[bench]
fn bench_parse_primary_expr_intconst(b: &mut Bencher) {
  b.iter(|| black_box(primary_expr("1 ")));
}

#[bench]
fn bench_parse_primary_expr_uintconst(b: &mut Bencher) {
  b.iter(|| black_box(primary_expr("1u ")));
}

#[bench]
fn bench_parse_primary_expr_floatconst(b: &mut Bencher) {
  b.iter(|| black_box(primary_expr("1.F ")));
}

#[bench]
fn bench_parse_primary_expr_doubleconst(b: &mut Bencher) {
  b.iter(|| black_box(primary_expr("1.LF ")));
}

#[bench]
fn bench_parse_primary_expr_boolconst(b: &mut Bencher) {
  b.iter(|| black_box(primary_expr("false")));
}

#[bench]
fn bench_parse_primary_expr_parens(b: &mut Bencher) {
  b.iter(|| black_box(primary_expr("(  (.0) )")));
}

#[bench]
fn bench_parse_postfix_function_call_no_args(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("vec3   (\nvoid\n) ;")));
}

#[bench]
fn bench_parse_postfix_function_call_one_arg(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("foo   (\n0\t\n) ;")));
}

#[bench]
fn bench_parse_postfix_function_call_multi_arg(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("foo   ( 0\t, false    ,\t\tbar) ;")));
}

#[bench]
fn bench_parse_postfix_expr_bracket(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("foo[\n  7354    ] ;")));
}

#[bench]
fn bench_parse_postfix_expr_dot(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("(foo).bar;")));
}

#[bench]
fn bench_parse_postfix_expr_dot_several(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("(foo.bar).zoo;")));
}

#[bench]
fn bench_parse_postfix_postinc(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("foo++;")));
}

#[bench]
fn bench_parse_postfix_postdec(b: &mut Bencher) {
  b.iter(|| black_box(postfix_expr("foo--;")));
}

#[bench]
fn bench_parse_unary_add(b: &mut Bencher) {
  b.iter(|| black_box(unary_expr("+foo;")));
}

#[bench]
fn bench_parse_unary_minus(b: &mut Bencher) {
  b.iter(|| black_box(unary_expr("-foo;")));
}

#[bench]
fn bench_parse_unary_not(b: &mut Bencher) {
  b.iter(|| black_box(unary_expr("!foo;")));
}

#[bench]
fn bench_parse_unary_complement(b: &mut Bencher) {
  b.iter(|| black_box(unary_expr("~foo;")));
}

#[bench]
fn bench_parse_unary_inc(b: &mut Bencher) {
  b.iter(|| black_box(unary_expr("++foo;")));
}

#[bench]
fn bench_parse_unary_dec(b: &mut Bencher) {
  b.iter(|| black_box(unary_expr("--foo;")));
}

#[bench]
fn bench_parse_expr_float(b: &mut Bencher) {
  b.iter(|| black_box(expr("314.LF;")));
}

#[bench]
fn bench_parse_expr_add_2(b: &mut Bencher) {
  b.iter(|| black_box(expr("(1 + 1);")));
}

#[bench]
fn bench_parse_expr_add_3(b: &mut Bencher) {
  b.iter(|| black_box(expr("((1u + 2u) + 3u)")));
}

#[bench]
fn bench_parse_expr_add_mult_3(b: &mut Bencher) {
  b.iter(|| black_box(expr("(1u * 2u) + 3u;")));
}

#[bench]
fn bench_parse_expr_add_sub_mult_div(b: &mut Bencher) {
  b.iter(|| black_box(expr("1 * (2 + 3) + 4 / (5 + 6);")));
}

#[bench]
fn bench_parse_complex_expr(b: &mut Bencher) {
  b.iter(|| black_box(expr("normalize((inverse(view) * vec4(ray.dir, 0.)).xyz);")));
}

#[bench]
fn bench_parse_function_identifier_typename(b: &mut Bencher) {
  b.iter(|| black_box(function_identifier("foo\n\t(")));
}

#[bench]
fn bench_parse_function_identifier_cast(b: &mut Bencher) {
  b.iter(|| black_box(function_identifier("vec3\t\n\n \t (")));
}

#[bench]
fn bench_parse_function_identifier_cast_array_unsized(b: &mut Bencher) {
  b.iter(|| black_box(function_identifier("vec3  [\t\n](")));
}

#[bench]
fn bench_parse_function_identifier_cast_array_sized(b: &mut Bencher) {
  b.iter(|| black_box(function_identifier("vec3  [\t 12\n](")));
}

#[bench]
fn bench_parse_void(b: &mut Bencher) {
  b.iter(|| black_box(void("void ")));
}

#[bench]
fn bench_parse_assignment_op_equal(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("= ")));
}

#[bench]
fn bench_parse_assignment_op_mult(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("*= ")));
}

#[bench]
fn bench_parse_assignment_op_div(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("/= ")));
}

#[bench]
fn bench_parse_assignment_op_mod(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("%= ")));
}

#[bench]
fn bench_parse_assignment_op_add(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("+= ")));
}

#[bench]
fn bench_parse_assignment_op_sub(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("-= ")));
}

#[bench]
fn bench_parse_assignment_op_lshift(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("<<= ")));
}

#[bench]
fn bench_parse_assignment_op_rshift(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op(">>= ")));
}

#[bench]
fn bench_parse_assignment_op_and(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("&= ")));
}

#[bench]
fn bench_parse_assignment_op_xor(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("^= ")));
}

#[bench]
fn bench_parse_assignment_op_or(b: &mut Bencher) {
  b.iter(|| black_box(assignment_op("|= ")));
}

#[bench]
fn bench_parse_expr_statement(b: &mut Bencher) {
  b.iter(|| black_box(expr_statement("foo\n\t=  \n314.f;")));
}

#[bench]
fn bench_parse_declaration_function_prototype(b: &mut Bencher) {
  b.iter(|| {
    black_box(declaration(
      "vec3 \nfoo ( vec2\n, out float \n\tthe_arg )\n;",
    ))
  });
}

#[bench]
fn bench_parse_declaration_init_declarator_list_single(b: &mut Bencher) {
  b.iter(|| black_box(declaration("int    \t  \nfoo =\t34  ;")));
}

#[bench]
fn bench_parse_declaration_init_declarator_list_complex(b: &mut Bencher) {
  b.iter(|| black_box(declaration("int    \t  \nfoo =\t34 \n,\tbar=      12\n ;")));
}

#[bench]
fn bench_parse_declaration_precision_low(b: &mut Bencher) {
  b.iter(|| black_box(declaration("precision lowp float;")));
}

#[bench]
fn bench_parse_declaration_precision_medium(b: &mut Bencher) {
  b.iter(|| black_box(declaration("precision mediump float;")));
}

#[bench]
fn bench_parse_declaration_precision_high(b: &mut Bencher) {
  b.iter(|| black_box(declaration("precision highp float;")));
}

#[bench]
fn bench_parse_declaration_uniform_block(b: &mut Bencher) {
  b.iter(|| black_box(declaration("uniform   \nUniformBlockTest\n {\n \t float   a  \n; \nvec3 b\n; foo \nc\n, \nd\n;\n }\n\t\n\t\t \t;")));
}

#[bench]
fn bench_parse_declaration_buffer_block(b: &mut Bencher) {
  b.iter(|| black_box(declaration("buffer   \nUniformBlockTest\n {\n \t float   a  \n; \nvec3 b   [   ]\n; foo \nc\n, \nd\n;\n }\n\t\n\t\t \t;")));
}

#[bench]
fn bench_parse_selection_statement_if(b: &mut Bencher) {
  b.iter(|| black_box(selection_statement("if \n(foo<10\n) \t{return false;}K")));
}

#[bench]
fn bench_parse_selection_statement_if_else(b: &mut Bencher) {
  b.iter(|| {
    black_box(selection_statement(
      "if \n(foo<10\n) \t{return 0.f\t;\n\n}\n else{\n\t return foo   ;}",
    ))
  });
}

#[bench]
fn bench_parse_switch_statement_empty(b: &mut Bencher) {
  b.iter(|| black_box(switch_statement("switch\n\n (  foo  \t   \n) { \n\n   }")));
}

#[bench]
fn bench_parse_switch_statement_cases(b: &mut Bencher) {
  b.iter(|| {
    black_box(switch_statement(
      "switch (foo) { case 0: case 1: return 12u; }",
    ))
  });
}

#[bench]
fn bench_parse_case_label_def(b: &mut Bencher) {
  b.iter(|| black_box(case_label("default   :")));
}

#[bench]
fn bench_parse_case_label(b: &mut Bencher) {
  b.iter(|| black_box(case_label("case\n\t 3   :")));
}

#[bench]
fn bench_parse_iteration_statement_while_empty(b: &mut Bencher) {
  b.iter(|| black_box(iteration_statement("while (  a >=\n\tb  )\t  {   \n}")));
}

#[bench]
fn bench_parse_iteration_statement_do_while_empty(b: &mut Bencher) {
  b.iter(|| {
    black_box(iteration_statement(
      "do \n {\n} while (  a >=\n\tb  )\t  \n;",
    ))
  });
}

#[bench]
fn bench_parse_iteration_statement_for_empty(b: &mut Bencher) {
  b.iter(|| {
    black_box(iteration_statement(
      "for\n\t (  \t\n\nfloat \ni \t=\n0.f\n;\ni\t<=  10.f; \n++i\n)\n{\n}",
    ))
  });
}

#[bench]
fn bench_parse_jump_continue(b: &mut Bencher) {
  b.iter(|| black_box(jump_statement("continue;")));
}

#[bench]
fn bench_parse_jump_break(b: &mut Bencher) {
  b.iter(|| black_box(jump_statement("break;")));
}

#[bench]
fn bench_parse_jump_return(b: &mut Bencher) {
  b.iter(|| black_box(jump_statement("return 3;")));
}

#[bench]
fn bench_parse_jump_empty_return(b: &mut Bencher) {
  b.iter(|| black_box(simple_statement("return;")));
}

#[bench]
fn bench_parse_jump_discard(b: &mut Bencher) {
  b.iter(|| black_box(jump_statement("discard;")));
}

#[bench]
fn bench_parse_simple_statement_return(b: &mut Bencher) {
  b.iter(|| black_box(simple_statement("return false;")));
}

#[bench]
fn bench_parse_compound_statement_empty(b: &mut Bencher) {
  b.iter(|| black_box(compound_statement("{}")));
}

#[bench]
fn bench_parse_compound_statement(b: &mut Bencher) {
  b.iter(|| {
    black_box(compound_statement(
      "{ if (true) {} isampler3D x; return 42 ; }",
    ))
  });
}

#[bench]
fn bench_parse_function_definition(b: &mut Bencher) {
  b.iter(|| {
    black_box(function_definition(
      "iimage2DArray \tfoo\n()\n \n{\n return \nbar\n;}",
    ))
  });
}

#[bench]
fn bench_parse_buffer_block_0(b: &mut Bencher) {
  let src = include_str!("../data/tests/buffer_block_0.glsl");
  b.iter(|| black_box(translation_unit(src)));
}

#[bench]
fn bench_parse_layout_buffer_block_0(b: &mut Bencher) {
  let src = include_str!("../data/tests/layout_buffer_block_0.glsl");
  b.iter(|| black_box(translation_unit(src)));
}

#[bench]
fn bench_parse_pp_space0(b: &mut Bencher) {
  b.iter(|| black_box(pp_space0("   \\\n  ")));
}

#[bench]
fn bench_parse_pp_version_number(b: &mut Bencher) {
  b.iter(|| black_box(pp_version_number("450")));
}

#[bench]
fn bench_parse_pp_version_profile(b: &mut Bencher) {
  b.iter(|| black_box(pp_version_profile("compatibility")));
}

#[bench]
fn bench_parse_pp_version(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#version 450 core\n")));
}

#[bench]
fn bench_parse_pp_version_newline(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#version 450 core\n")));
}

#[bench]
fn bench_parse_pp_define(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#define test123 .0f\n")));
}

#[bench]
fn bench_parse_pp_define_with_args(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#define \\\n add(  x, y  ) \\\n (x + y)")));
}

#[bench]
fn bench_parse_pp_define_multiline(b: &mut Bencher) {
  b.iter(|| {
    black_box(preprocessor(
      r#"#define foo \
       32"#,
    ))
  });
}

#[bench]
fn bench_parse_pp_else(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#    else\n")));
}

#[bench]
fn bench_parse_pp_elif(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#   elif \\\n42\n")));
}

#[bench]
fn bench_parse_pp_endif(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#\\\nendif")));
}

#[bench]
fn bench_parse_pp_error(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#error \\\n     some message")));
}

#[bench]
fn bench_parse_pp_if(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("# \\\nif 42")));
}

#[bench]
fn bench_parse_pp_ifdef(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#ifdef       FOO\n")));
}

#[bench]
fn bench_parse_pp_ifndef(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#\\\nifndef \\\n   FOO\n")));
}

#[bench]
fn bench_parse_pp_include(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#include \\\n\"filename\"\n")));
}

#[bench]
fn bench_parse_pp_line(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#line 2 \\\n 4\n")));
}

#[bench]
fn bench_parse_pp_pragma(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#\\\npragma  some   flag")));
}

#[bench]
fn bench_parse_pp_undef(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("# undef \\\n FOO")));
}

#[bench]
fn bench_parse_pp_extension_name(b: &mut Bencher) {
  b.iter(|| black_box(pp_extension_name("GL_foobar_extension ")));
}

#[bench]
fn bench_parse_pp_extension_behavior(b: &mut Bencher) {
  b.iter(|| black_box(pp_extension_behavior("disable")));
}

#[bench]
fn bench_parse_pp_extension(b: &mut Bencher) {
  b.iter(|| black_box(preprocessor("#extension all: require\n")));
}

#[bench]
fn bench_parse_dot_field_expr_array(b: &mut Bencher) {
  b.iter(|| black_box(expr("a[0].xyz;")));
}

#[bench]
fn bench_parse_dot_field_expr_statement(b: &mut Bencher) {
  b.iter(|| {
    black_box(statement(
      "vec3 v = smoothstep(vec3(border_width), vec3(0.0), v_barycenter).zyx;",
    ))
  });
}

#[bench]
fn bench_parse_arrayed_identifier(b: &mut Bencher) {
  b.iter(|| black_box(arrayed_identifier("foo \t\n  [\n\t ]")));
}

#[bench]
fn bench_parse_nested_parens(b: &mut Bencher) {
  b.iter(|| parens_expr("((((((((1.0f))))))))"));
}
