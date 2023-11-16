mod warp_controller;
mod warp_job_account;
mod warp_job_account_tracker;
mod warp_legacy_account;
mod warp_resolver;
mod warp_templates;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
