/// Sanitization des logs pour OPSEC
///
/// TM-006 Fix: Empêche la corrélation via logs complets

/// Sanitize un UUID pour les logs
///
/// Format: "abc12345...90ef" (8 premiers + 4 derniers chars)
///
/// Empêche la corrélation complète tout en permettant le debug
pub fn sanitize_uuid(uuid: &uuid::Uuid) -> String {
    let uuid_str = uuid.to_string();
    if uuid_str.len() < 12 {
        return "<invalid-uuid>".to_string();
    }
    format!("{}...{}", &uuid_str[..8], &uuid_str[uuid_str.len()-4..])
}

/// Sanitize une adresse Monero pour les logs
///
/// Format: "9w...XYZ" (2 premiers + 3 derniers chars)
///
/// Les 2 premiers chars identifient le network (9 = mainnet, A = testnet)
/// Les 3 derniers permettent de différencier les addresses en debug
pub fn sanitize_address(address: &str) -> String {
    if address.len() < 6 {
        return "<invalid-address>".to_string();
    }
    format!("{}...{}", &address[..2], &address[address.len()-3..])
}

/// Sanitize un montant XMR (optionnel, si vraiment paranoid)
///
/// Arrondit à 2 décimales pour empêcher l'identification exacte
pub fn sanitize_amount(piconeros: u64) -> String {
    let xmr = piconeros as f64 / 1_000_000_000_000.0;
    format!("~{:.2} XMR", xmr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_sanitize_uuid() {
        let uuid = Uuid::parse_str("abc12345-6789-0123-4567-890ef1234567").unwrap();
        let sanitized = sanitize_uuid(&uuid);

        assert_eq!(sanitized, "abc12345...4567");
        assert!(!sanitized.contains("6789")); // Partie du milieu cachée
    }

    #[test]
    fn test_sanitize_address() {
        let addr = "9wHq7XM8ZtKpVqnEQB8X...ABCXYZ";
        let sanitized = sanitize_address(addr);

        assert_eq!(sanitized, "9w...XYZ");
        assert!(!sanitized.contains("Hq7XM8")); // Milieu caché
    }

    #[test]
    fn test_sanitize_amount() {
        // 1 XMR = 1_000_000_000_000 piconeros
        let amount = 1_234_567_890_123;
        let sanitized = sanitize_amount(amount);

        assert_eq!(sanitized, "~1.23 XMR");
    }
}
