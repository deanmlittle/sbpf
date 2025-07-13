use std::str::FromStr;

pub fn evaluate_constant_expression(expr: &str) -> Result<String, String> {

    let mut tokens = Vec::new(); // let mut tokens = vec![];

    let mut cur_token = String::new();
    for c in expr.chars() {
        if c.is_alphanumeric() || c == '_' {
            cur_token.push(c);
        } else if c == '+' || c == '-' {
            if !cur_token.is_empty() {
                tokens.push(cur_token.clone());
                cur_token.clear();
            }
            tokens.push(c.to_string());
        } else if c == ' ' {
            continue;
        } else {
            return Err(format!("Invalid character in expression: {}", c));
        }
    }

    if !cur_token.is_empty() {
        tokens.push(cur_token.clone());
    }

    let mut result_tokens = Vec::<String>::new();
    let mut i = 0;
    let mut constant = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "+" | "-" => {
                if i + 1 < tokens.len() {
                    if let Ok(num) = i32::from_str(&tokens[i + 1]) {
                        if tokens[i] == "+" {
                            constant += num;
                        } else {
                            constant -= num;
                        }
                        i += 2;
                    } else {
                        return Err(format!("Invalid token after {}: {}", tokens[i], tokens[i + 1]));
                    }
                } else {
                    return Err(format!("Operator {} has no operand", tokens[i]));
                }
            }
            token => {
                if let Ok(num) = i32::from_str(token) {
                    constant += num;
                } else {
                    result_tokens.push(token.to_string());
                }
            }
        }
        i += 1;
    }

    if constant != 0 {
        result_tokens.push(constant.to_string());
    }

    Ok(result_tokens.join(if constant > 0 { "+" } else { "-" }))
}