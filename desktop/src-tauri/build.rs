fn main() {
  println!("cargo:rerun-if-changed=../.env");
  let _ = dotenvy::from_filename("../.env");
  
  if let Ok(salt) = std::env::var("WISP_SALT") {
      println!("cargo:rustc-env=WISP_SALT={}", salt);
  }
  if let Ok(api_base) = std::env::var("VITE_API_BASE") {
      println!("cargo:rustc-env=VITE_API_BASE={}", api_base);
  }
  if let Ok(api_domain) = std::env::var("VITE_API_DOMAIN") {
      println!("cargo:rustc-env=VITE_API_DOMAIN={}", api_domain);
  }
  if let Ok(upstream_proxy) = std::env::var("VITE_UPSTREAM_PROXY") {
      println!("cargo:rustc-env=VITE_UPSTREAM_PROXY={}", upstream_proxy);
  }

  tauri_build::build()
}
