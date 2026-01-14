use anyhow::Result;

pub async fn save_api_key(service: &str, key: &str) -> Result<()> {
    use keyring::Entry;

    let entry = Entry::new("memflow", service)?;
    entry.set_password(key)?;

    Ok(())
}

pub async fn get_api_key(service: &str) -> Result<Option<String>> {
    use keyring::Entry;

    let entry = Entry::new("memflow", service)?;

    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("获取密钥失败: {}", e)),
    }
}

pub async fn delete_api_key(service: &str) -> Result<()> {
    use keyring::Entry;

    let entry = Entry::new("memflow", service)?;
    entry.delete_password()?;

    Ok(())
}
