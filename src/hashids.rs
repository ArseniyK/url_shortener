use harsh::Harsh;

pub async fn configure() -> Harsh {
  let salt:String = std::env::var("HASHID_SALT").expect("HASHID_SALT env var needs to be set");
  let min_length:usize = std::env::var("HASHID_MIN_LENGTH").expect("HASHID_MIN_LENGTH env var needs to be set").parse().unwrap();
  Harsh::builder().salt(salt).length(min_length).build().expect("Error during Harsh configure")
}
