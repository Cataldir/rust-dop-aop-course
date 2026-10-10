fn main()->anyhow::Result<()> {
    let tenant=shared_kernel::Tenant::new(std::env::args().nth(1).unwrap_or_else(||"demo".into()))?;
    let secret=std::env::var("COURSE_JWT_SECRET")?;if secret.len()<32{anyhow::bail!("secret too short");}
    let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as usize;
    let token=jsonwebtoken::encode(&jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),&axum_api::Claims{sub:"learner".into(),tenant,exp:now+3600,iss:"course-local".into(),aud:"course-api".into()},&jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()))?;
    println!("{token}");Ok(())
}