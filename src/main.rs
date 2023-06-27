use std::sync::Arc;

use nom::IResult;
use nom_supreme::error::ErrorTree;
use nom_supreme::tag::complete::tag;

pub type AsmResult<'a, 'b, O> = IResult<&'a str, O, ErrorTree<&'b str>>;

pub fn create_test_parser(
    captured_tag: Arc<&str>,
) -> impl FnMut(&str) -> AsmResult<(String, String)> + '_ {
    move |i: &str| {
        //
        // Here be dragons
        //
        // Replace "nom::bytes::complete::tag" with "nom_supreme::tag::complete::tag" and get
        // "error: lifetime may not live long enough" error
        //     --> src/main.rs:10:5
        //     |
        //  8  |       captured_tag: &str,
        //     |                     - let's call the lifetime of this reference `'1`
        //  9  |   ) -> impl FnMut(&str) -> AsmResult<(String, String)> + '_ {
        //  10 | /     move |i: &str| {
        //  11 | |         //
        //  12 | |         // Here be dragons
        //  13 | |         //
        //  ...  |
        //  22 | |         Ok((i, (String::from(parsed_tag), String::from(parsed_suffix))))
        //  23 | |     }
        //     | |_____^ returning this value requires that `'1` must outlive `'static`
        //
        // I've tried:
        // 1. Cloning "captured_tag" -> "using 'clone' on a double reference"
        // 2. captured_tag.to_owned() -> "returns a value referencing data owned by the current function"
        // 3. Cloning "captured_tag" in outer scope -> same lifetime error
        // 4. captured_tag.to_owned() in outer scope -> "captured variable cannot escape FnMut"
        // 5. Using "Arc", this works! but why do I need to resort to higher level memory management when the standard nom tag function works?
        let captured_tag_parser = nom::bytes::complete::tag(*captured_tag);
        let (i, parsed_tag) = captured_tag_parser(i)?;

        // This is fine because the string is owned by this closure
        let suffix_parser = tag("_SOME_SUFFIX");
        let (i, parsed_suffix) = suffix_parser(i)?;

        Ok((i, (String::from(parsed_tag), String::from(parsed_suffix))))
    }
}

fn main() {
    let mut parser = create_test_parser(Arc::new("something"));
    let first_example_success = parser("something_SOME_SUFFIX").is_ok();
    let second_example_success = parser("bad").is_ok();
    println!("first_example_success: {first_example_success} (expected true), second_example_success: {second_example_success} (expected false)");
}
