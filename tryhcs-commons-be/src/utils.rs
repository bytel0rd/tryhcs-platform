use rand::Rng;

pub fn mask_email(email: &str) -> String {
    if let Some((local, domain)) = email.split_once('@') {
        let masked_local = if local.len() > 2 {
            format!(
                "{}{}{}",
                &local.chars().next().unwrap(),
                "*".repeat(local.len() - 2),
                &local.chars().last().unwrap()
            )
        } else {
            "*".repeat(local.len())
        };
        format!("{}@{}", masked_local, domain)
    } else {
        email.to_string()
    }
}

pub fn mask_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();
    if digits.len() < 4 {
        return "*".repeat(digits.len());
    }
    let masked_part = "*".repeat(digits.len() - 4);
    let last_four = &digits[digits.len() - 4..];
    format!("{}{}", masked_part, last_four)
}

pub fn generate_password(length: usize) -> String {
    let charset: Vec<char> = r#"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;:'",.<>?/"#.chars().collect();
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| charset[rng.gen_range(0..charset.len())])
        .collect()
}

pub fn generate_otp(length: u32) -> eyre::Result<String> {
    return Ok("12345".to_owned());
}
