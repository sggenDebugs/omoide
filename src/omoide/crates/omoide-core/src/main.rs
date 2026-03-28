use omoide_core::{mlock_secret, security::suppress_core_dumps};
use omoide_crypto::MasterKey;

fn main() {
    // T1: must be first — before any secret is allocated
    suppress_core_dumps();
    println!("[security] core dump suppression applied");

    let key = MasterKey::new_zeroed();
    if mlock_secret(&key){
        println!("[security] swap protection applied");
    }
    // future: vault unlock, UI init, etc. all go below this line
}