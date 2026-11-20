#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    ApiKey,
    Secret,
    Serial,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::ApiKey, Kind::Secret, Kind::Serial];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::ApiKey => "api_key",
            Kind::Secret => "secret",
            Kind::Serial => "serial",
        }
    }

    pub fn from_str(s: &str) -> Option<Kind> {
        match s {
            "api_key" => Some(Kind::ApiKey),
            "secret" => Some(Kind::Secret),
            "serial" => Some(Kind::Serial),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::ApiKey => "API Key",
            Kind::Secret => "密钥",
            Kind::Serial => "序列号",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub name: String,
    pub kind: Kind,
    pub value: String,
    pub service: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct EntryDraft {
    pub name: String,
    pub kind: Kind,
    pub value: String,
    pub service: String,
    pub notes: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_str_roundtrip() {
        for k in Kind::ALL {
            assert_eq!(Kind::from_str(k.as_str()), Some(k));
        }
    }

    #[test]
    fn kind_from_str_unknown_is_none() {
        assert_eq!(Kind::from_str("bogus"), None);
    }

    #[test]
    fn kind_labels_are_chinese() {
        assert_eq!(Kind::ApiKey.label(), "API Key");
        assert_eq!(Kind::Secret.label(), "密钥");
        assert_eq!(Kind::Serial.label(), "序列号");
    }

    #[test]
    fn draft_default_kind_is_api_key() {
        assert_eq!(EntryDraft::default().kind, Kind::ApiKey);
    }
}