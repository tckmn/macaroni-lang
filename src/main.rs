extern crate macaroni_lang;
use macaroni_lang::*;

fn main() {
    let mut mac = macaroni::Macaroni::new();
    for test in r#"
            print tobase add 1 1 10
            print tobase multiply 5 5 10
            print tobase pow 2 16 10
            print tobase pow 2 -1 10
            print tobase multiply pow 2 -1 100 10
            print tobase multiply floor pow 2 -1 100 10
            print wrap 33
            print tobase rand 10
            print "Hello, World!"
            print tobase asdf 10 print " = 0"
            :a add 1 1 print tobase a 10
            print cat "foo" "bar"
            "#.lines() {
        if !test.chars().all(char::is_whitespace) {
            mac.run(test.to_string());
            println!("");
        }
    }
}
