extern crate macaroni_lang;
use macaroni_lang::*;

fn main() {
    let mac = macaroni::Macaroni::new();
    mac.run("print tobase add 1 1 10 print \"\n\"".to_string());
    mac.run("print tobase multiply 5 5 10 print \"\n\"".to_string());
    mac.run("print tobase pow 2 16 10 print \"\n\"".to_string());
    mac.run("print tobase pow 2 -1 10 print \"\n\"".to_string());
    mac.run("print tobase multiply pow 2 -1 100 10 print \"\n\"".to_string());
    mac.run("print tobase multiply floor pow 2 -1 100 10 print \"\n\"".to_string());
    mac.run("print wrap 33 print wrap 10".to_string());
    mac.run("print tobase rand 10 print \"\n\"".to_string());
    mac.run("print \"Hello, World!\n\"".to_string());
}
