pub fn check_difficulty(hash: &[u8; 64], fine_difficulty: u8) -> bool {
    let leading_zeros = hash.iter().take_while(|&&b| b == 0).count();
    leading_zeros >= fine_difficulty as usize
}

pub fn adjust_difficulty(
    current_tier: u8,
    current_fine: u8,
    actual_time: u64,
    target_time: u64,
) -> (u8, u8) {
    if actual_time < target_time / 2 {
        if current_fine < 255 {
            (current_tier, current_fine + 1)
        } else {
            (current_tier.saturating_add(1), 0)
        }
    } else if actual_time > target_time * 2 {
        if current_fine > 0 {
            (current_tier, current_fine - 1)
        } else {
            (current_tier.saturating_sub(1), 255)
        }
    } else {
        (current_tier, current_fine)
    }
}
