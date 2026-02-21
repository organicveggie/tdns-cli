pub fn make_domain_name(domain: &str, zone: &str) -> String {
    if domain.ends_with(zone) {
        return domain.to_string();
    }

    if domain.ends_with('.') {
        format!("{}{}", domain, zone)
    } else if domain.is_empty() {
        zone.to_string()
    } else {
        format!("{}.{}", domain, zone)
    }
}

pub fn get_target_domain(zone: &str, domain: &Option<String>) -> String {
    if let Some(domain_name) = domain {
        if domain_name.ends_with(zone) {
            return domain_name.to_string();
        }
        return format!("{}.{}", domain_name, zone);
    }

    zone.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MakeDomainNameTestCase {
        domain: String,
        zone: String,
        expected: String,
    }

    #[test]
    fn make_domain_name_test() {
        let cases = vec![
            MakeDomainNameTestCase {
                domain: "".to_string(),
                zone: "example.com".to_string(),
                expected: "example.com".to_string(),
            },
            MakeDomainNameTestCase {
                domain: "www".to_string(),
                zone: "example.com".to_string(),
                expected: "www.example.com".to_string(),
            },
            MakeDomainNameTestCase {
                domain: "www.example.com".to_string(),
                zone: "example.com".to_string(),
                expected: "www.example.com".to_string(),
            },
            MakeDomainNameTestCase {
                domain: "www.".to_string(),
                zone: "example.com".to_string(),
                expected: "www.example.com".to_string(),
            },
            MakeDomainNameTestCase {
                domain: "www.sub".to_string(),
                zone: "example.com".to_string(),
                expected: "www.sub.example.com".to_string(),
            },
            MakeDomainNameTestCase {
                domain: "www.sub.".to_string(),
                zone: "example.com".to_string(),
                expected: "www.sub.example.com".to_string(),
            },
        ];
        for tc in cases {
            assert_eq!(make_domain_name(&tc.domain, &tc.zone), tc.expected);
        }
    }
}
