use crate::ContractError;
use cosmwasm_std::StdError;
use json_codec_wasm::ast::Ref;

// supports custom jsonpath-like expression with options for indexing
// - fields - $.field
// - array entries - $.field[0]
// - multiple nested fields / entries - $.field1.field2[0].field3
pub fn resolve_path(r: Ref, path: String) -> Result<Ref, ContractError> {
    let mut obj = r;
    let mut curr = String::new();
    let mut idx = 0;

    while idx < path.len() {
        let co = path.chars().nth(idx);

        if let Some(c) = co {
            match c {
                '$' => {
                    idx += 1;
                }
                '[' => {
                    // field[0] case
                    if !curr.is_empty() {
                        obj = obj.get(curr);
                        curr = String::new();
                    }

                    let (_, array_idx) = read_array_index(path, idx + 1)?;
                    obj = obj.at(array_idx);
                    break;
                }
                '.' => {
                    // $ case
                    if curr.is_empty() {
                        idx += 1;
                        continue;
                    }

                    obj = obj.get(curr);
                    curr = String::new();
                    idx += 1;
                }
                c => {
                    curr.push(c);
                    idx += 1;
                }
            }
        } else {
            return Err(ContractError::ResolveError {});
        }
    }

    if !curr.is_empty() {
        obj = obj.get(curr);
    }

    Ok(obj)
}

fn read_array_index(path: String, from: usize) -> Result<(usize, usize), ContractError> {
    let mut idx = from;
    let mut curr = String::new();
    while path.chars().nth(idx).unwrap() != ']' {
        curr.push(
            path.chars()
                .nth(idx)
                .ok_or_else(|| StdError::generic_err("Array indexing error"))?,
        );
        idx += 1;
    }

    Ok((idx + 1, str::parse::<usize>(curr.as_str())?))
}
