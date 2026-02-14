pub fn get_target_domain(zone: &str, domain: &Option<String>) -> String {
    if let Some(domain_name) = domain {
        if domain_name.ends_with(zone) {
            return domain_name.to_string();
        }
        return format!("{}.{}", domain_name, zone);
    }

    zone.to_string()
}
