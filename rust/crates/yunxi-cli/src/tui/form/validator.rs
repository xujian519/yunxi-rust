use std::fmt;

pub trait Validator<T>: fmt::Debug + Send + Sync {
    fn validate(&self, value: &T) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct RequiredValidator;

impl<T> Validator<T> for RequiredValidator
where
    T: AsRef<str>,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        if value.as_ref().trim().is_empty() {
            Err("此字段不能为空".to_string())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct LengthValidator {
    min: usize,
    max: usize,
}

impl LengthValidator {
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }
}

impl<T> Validator<T> for LengthValidator
where
    T: AsRef<str>,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        let len = value.as_ref().len();
        if len < self.min {
            Err(format!("长度不能少于 {} 个字符", self.min))
        } else if len > self.max {
            Err(format!("长度不能超过 {} 个字符", self.max))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct RangeValidator<T> {
    min: T,
    max: T,
}

impl<T> RangeValidator<T>
where
    T: PartialOrd + Clone,
{
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> Validator<T> for RangeValidator<T>
where
    T: PartialOrd + Clone + fmt::Display + Send + Sync + std::fmt::Debug,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        if *value < self.min {
            Err(format!("值不能小于 {}", self.min))
        } else if *value > self.max {
            Err(format!("值不能大于 {}", self.max))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternValidator {
    pattern: String,
    message: String,
}

impl PatternValidator {
    pub fn new(pattern: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            message: message.into(),
        }
    }
}

impl<T> Validator<T> for PatternValidator
where
    T: AsRef<str>,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        let regex = match regex::Regex::new(&self.pattern) {
            Ok(r) => r,
            Err(_) => return Err("正则表达式格式错误".to_string()),
        };

        if regex.is_match(value.as_ref()) {
            Ok(())
        } else {
            Err(self.message.clone())
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailValidator;

impl<T> Validator<T> for EmailValidator
where
    T: AsRef<str>,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        let email = value.as_ref();
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| "邮箱验证正则表达式错误".to_string())?;

        if email_regex.is_match(email) {
            Ok(())
        } else {
            Err("请输入有效的邮箱地址".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct URLValidator;

impl<T> Validator<T> for URLValidator
where
    T: AsRef<str>,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        let url = value.as_ref();
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(())
        } else {
            Err("请输入有效的 URL (以 http:// 或 https:// 开头)".to_string())
        }
    }
}

pub struct CustomValidator<T>
where
    T: Send + Sync,
{
    validator: Box<dyn Fn(&T) -> Result<(), String> + Send + Sync>,
}

impl<T> CustomValidator<T>
where
    T: Send + Sync,
{
    pub fn new<F>(validator: F) -> Self
    where
        F: Fn(&T) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            validator: Box::new(validator),
        }
    }
}

impl<T> std::fmt::Debug for CustomValidator<T>
where
    T: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomValidator").finish()
    }
}

impl<T> Validator<T> for CustomValidator<T>
where
    T: Send + Sync + std::fmt::Debug,
{
    fn validate(&self, value: &T) -> Result<(), String> {
        (self.validator)(value)
    }
}

pub struct ValidatorSet<T> {
    validators: Vec<Box<dyn Validator<T> + Send + Sync>>,
}

impl<T> Default for ValidatorSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ValidatorSet<T> {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    pub fn add<V>(mut self, validator: V) -> Self
    where
        V: Validator<T> + 'static + Send + Sync,
    {
        self.validators.push(Box::new(validator));
        self
    }

    pub fn validate(&self, value: &T) -> Result<(), String> {
        for validator in &self.validators {
            validator.validate(value)?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }
}

impl<T> fmt::Debug for ValidatorSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatorSet")
            .field("validator_count", &self.validators.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validator() {
        let validator = RequiredValidator;

        assert!(validator.validate(&"hello").is_ok());
        assert!(validator.validate(&"  hello  ").is_ok());
        assert!(validator.validate(&"").is_err());
        assert!(validator.validate(&"   ").is_err());
    }

    #[test]
    fn test_length_validator() {
        let validator = LengthValidator::new(3, 10);

        assert!(validator.validate(&"abc").is_ok());
        assert!(validator.validate(&"abcdefghij").is_ok());
        assert!(validator.validate(&"ab").is_err());
        assert!(validator.validate(&"abcdefghijk").is_err());
    }

    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::new(1i32, 100i32);

        assert!(validator.validate(&50i32).is_ok());
        assert!(validator.validate(&1i32).is_ok());
        assert!(validator.validate(&100i32).is_ok());
        assert!(validator.validate(&0i32).is_err());
        assert!(validator.validate(&101i32).is_err());
    }

    #[test]
    fn test_pattern_validator() {
        let validator = PatternValidator::new(r"^\d+$", "请输入数字");

        assert!(validator.validate(&"123").is_ok());
        assert!(validator.validate(&"456").is_ok());
        assert!(validator.validate(&"abc").is_err());
        assert!(validator.validate(&"12a").is_err());
    }

    #[test]
    fn test_email_validator() {
        let validator = EmailValidator;

        assert!(validator.validate(&"user@example.com").is_ok());
        assert!(validator.validate(&"test.user@domain.co.uk").is_ok());
        assert!(validator.validate(&"invalid").is_err());
        assert!(validator.validate(&"@example.com").is_err());
        assert!(validator.validate(&"user@").is_err());
    }

    #[test]
    fn test_url_validator() {
        let validator = URLValidator;

        assert!(validator.validate(&"http://example.com").is_ok());
        assert!(validator.validate(&"https://example.com").is_ok());
        assert!(validator.validate(&"https://example.com/path").is_ok());
        assert!(validator.validate(&"ftp://example.com").is_err());
        assert!(validator.validate(&"example.com").is_err());
    }

    #[test]
    fn test_custom_validator() {
        let validator = CustomValidator::new(|value: &i32| {
            if value % 2 == 0 {
                Ok(())
            } else {
                Err("必须是偶数".to_string())
            }
        });

        assert!(validator.validate(&2).is_ok());
        assert!(validator.validate(&4).is_ok());
        assert!(validator.validate(&1).is_err());
        assert!(validator.validate(&3).is_err());
    }

    #[test]
    fn test_validator_set_creation() {
        let set = ValidatorSet::<String>::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_validator_set_add() {
        let set = ValidatorSet::<String>::new()
            .add(RequiredValidator)
            .add(LengthValidator::new(3, 10));

        assert_eq!(set.len(), 2);
        assert!(!set.is_empty());
    }

    #[test]
    fn test_validator_set_validate_all() {
        let set = ValidatorSet::<String>::new()
            .add(RequiredValidator)
            .add(LengthValidator::new(3, 10));

        let v1 = "hello".to_string();
        let v2 = "hi".to_string();
        let v3 = String::new();
        assert!(set.validate(&v1).is_ok());
        assert!(set.validate(&v2).is_err());
        assert!(set.validate(&v3).is_err());
    }

    #[test]
    fn test_validator_set_validate_fail_fast() {
        let set = ValidatorSet::<String>::new()
            .add(RequiredValidator)
            .add(LengthValidator::new(3, 10));

        let v = String::new();
        let result = set.validate(&v);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能为空"));
    }

    #[test]
    fn test_pattern_validator_invalid_regex() {
        let validator = PatternValidator::new(r"[invalid", "错误");

        assert!(validator.validate(&"test").is_err());
    }

    #[test]
    fn test_custom_validator_complex() {
        let validator = CustomValidator::new(|value: &String| {
            if value.chars().all(|c| c.is_ascii_uppercase()) {
                Ok(())
            } else {
                Err("必须全部是大写字母".to_string())
            }
        });

        assert!(validator.validate(&"HELLO".to_string()).is_ok());
        assert!(validator.validate(&"Hello".to_string()).is_err());
        assert!(validator.validate(&"hello".to_string()).is_err());
    }

    #[test]
    fn test_multiple_validators_chain() {
        let set = ValidatorSet::<String>::new()
            .add(RequiredValidator)
            .add(LengthValidator::new(5, 20))
            .add(PatternValidator::new(r"^[a-zA-Z]+$", "只能包含字母"))
            .add(EmailValidator);

        assert!(set.validate(&"user@example.com".to_string()).is_ok());
        assert!(set.validate(&"ab@cd.com".to_string()).is_err());
        assert!(set.validate(&String::new()).is_err());
    }

    #[test]
    fn test_range_validator_float() {
        let validator = RangeValidator::new(0.0f64, 1.0f64);

        assert!(validator.validate(&0.5f64).is_ok());
        assert!(validator.validate(&0.0f64).is_ok());
        assert!(validator.validate(&1.0f64).is_ok());
        assert!(validator.validate(&-0.1f64).is_err());
        assert!(validator.validate(&1.1f64).is_err());
    }

    #[test]
    fn test_validator_set_debug() {
        let set = ValidatorSet::<String>::new()
            .add(RequiredValidator)
            .add(LengthValidator::new(3, 10));

        let debug_str = format!("{:?}", set);
        assert!(debug_str.contains("ValidatorSet"));
        assert!(debug_str.contains("validator_count"));
    }
}
