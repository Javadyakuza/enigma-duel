use cosmwasm_std::to_binary;

pub fn create_key_hash(con_1: String, con_2: String) -> String {
    to_binary(&format!("{}{}", con_1, con_2))
        .unwrap()
        .to_string()
}
