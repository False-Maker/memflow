// Sample Rust Code for OCR Testing
// This simulates code that would be extracted from terminal screenshots

fn main() {
    println!("Hello, world!");
}

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

#[derive(Debug, serde::Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

impl User {
    fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }
    
    fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sum() {
        assert_eq!(calculate_sum(2, 3), 5);
    }
    
    #[test]
    fn test_user_validation() {
        let user = User::new(1, "test".to_string(), "test@example.com".to_string());
        assert!(user.validate_email());
    }
}

// Example async function
async fn fetch_user_data(user_id: u64) -> Result<User, Box<dyn std::error::Error>> {
    // This would normally make an HTTP request
    let user = User::new(
        user_id,
        format!("user_{}", user_id),
        format!("user_{}@example.com", user_id),
    );
    Ok(user)
}