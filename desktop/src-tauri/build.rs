fn main() {
  let _ = dotenvy::from_filename("../.env");
  
  // Export variables from .env to the Rust compiler environment
  if let Ok(salt) = std::env::var("WISP_SALT") {
      println!("cargo:rustc-env=WISP_SALT={}", salt);
  }
  if let Ok(proxy_port) = std::env::var("VITE_LOCAL_PROXY_PORT") {
      println!("cargo:rustc-env=VITE_LOCAL_PROXY_PORT={}", proxy_port);
  }
  if let Ok(api_domain) = std::env::var("VITE_API_DOMAIN") {
      println!("cargo:rustc-env=VITE_API_DOMAIN={}", api_domain);
  }
  if let Ok(upstream_proxy) = std::env::var("VITE_UPSTREAM_PROXY") {
      println!("cargo:rustc-env=VITE_UPSTREAM_PROXY={}", upstream_proxy);
  }

  tauri_build::build()
}
