use holochain_p2p::kitsune_p2p::dependencies::kitsune_p2p_types::dependencies::lair_keystore_api::LairResult;

pub async fn read_piped_passphrase() -> LairResult<sodoken::BufRead> {
    use std::io::Read;
    Ok(tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        let passphrase = <sodoken::BufWriteSized<512>>::new_mem_locked()?;
        let mut next_char = 0;
        loop {
            let mut lock = passphrase.write_lock();
            let done = match stdin.read_exact(&mut lock[next_char..next_char + 1]) {
                Ok(_) => {
                    if lock[next_char] == 10 {
                        true
                    } else {
                        next_char += 1;
                        false
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => true,
                Err(e) => return Err(e),
            };
            if done {
                if next_char == 0 {
                    return Ok(sodoken::BufWrite::new_no_lock(0).to_read());
                }
                if lock[next_char - 1] == 13 {
                    next_char -= 1;
                }
                let out = sodoken::BufWrite::new_mem_locked(next_char)?;
                {
                    let mut out_lock = out.write_lock();
                    out_lock.copy_from_slice(&lock[..next_char]);
                }
                return Ok(out.to_read());
            }
        }
    })
    .await
    .map_err(one_err::OneErr::new)??)
}
