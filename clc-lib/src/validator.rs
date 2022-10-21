pub fn is_valid_name(ident: &str) -> bool{
    for c in ident.chars() {
        if !(('0'..'9').contains(&c) || ('a'..'z').contains(&c) || ('A'..'Z').contains(&c) || "_-.~#".contains(c)){
            return false
        }
    }
    true
}