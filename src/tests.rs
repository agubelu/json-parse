#[cfg(test)]
use crate::*;
#[test]
fn it_works() {
    let json = r#"[1,2,3,
        
        
        
        
        ?]"#;
    let res = parse(json);
    println!("{:?}", res);
    if let Err(ParseError { msg, .. }) = &res {
        println!("{msg}");
    }

    if let Ok(JsonElement::String(s)) = &res {
        println!("{s}");
    }
}
