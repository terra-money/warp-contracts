#[cfg(feature = "interface")]
mod warp_controller;
#[cfg(feature = "interface")]
mod warp_job_account;
#[cfg(feature = "interface")]
mod warp_job_account_tracker;
#[cfg(feature = "interface")]
mod warp_legacy_account;
#[cfg(feature = "interface")]
mod warp_resolver;
#[cfg(feature = "interface")]
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
