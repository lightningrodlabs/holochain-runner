use std::io::Result;
use std::sync::{Arc, Mutex};

pub type SharedLockedArray = Arc<Mutex<sodoken::LockedArray>>;

pub fn read_piped_passphrase() -> Result<SharedLockedArray> {
    use std::io::Read;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut passphrase = sodoken::SizedLockedArray::<512>::new()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut next_char = 0;
    loop {
        let mut lock = passphrase.lock();
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
                return Ok(Arc::new(Mutex::new(sodoken::LockedArray::from(
                    Vec::<u8>::new(),
                ))));
            }
            if lock[next_char - 1] == 13 {
                next_char -= 1;
            }
            let mut out = sodoken::LockedArray::new(next_char)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            {
                let mut out_lock = out.lock();
                out_lock.copy_from_slice(&lock[..next_char]);
            }
            return Ok(Arc::new(Mutex::new(out)));
        }
    }
}

pub fn passphrase_from_bytes(bytes: Vec<u8>) -> SharedLockedArray {
    Arc::new(Mutex::new(sodoken::LockedArray::from(bytes)))
}
